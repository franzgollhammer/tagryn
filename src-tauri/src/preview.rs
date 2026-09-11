use crate::{files, service::Service};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{ImageDecoder, ImageReader};
use std::{io::Cursor, path::Path, sync::Arc};

pub async fn preview(service: Arc<Service>, id: &str) -> Result<Option<String>, String> {
    let file = service.file(id)?;
    let current = files::entry(Path::new(&file.path), id.into())?;
    let key = format!("{}:{}:{}", file.path, current.size, current.modified);
    if let Some(cached) = service
        .previews
        .lock()
        .map_err(|e| e.to_string())?
        .get(&key)
        .cloned()
    {
        return Ok(Some(cached));
    }
    let _permit = service
        .preview_slots
        .acquire()
        .await
        .map_err(|e| e.to_string())?;
    let path = file.path.clone();
    let ext = files::extension(Path::new(&path));
    let bytes = if ["jpg", "jpeg", "png", "webp", "tif", "tiff"].contains(&ext.as_str()) {
        if current.size > 256 * 1024 * 1024 {
            return Ok(None);
        }
        let path = path.clone();
        tokio::task::spawn_blocking(move || std::fs::read(&path).map_err(|e| e.to_string()))
            .await
            .map_err(|e| e.to_string())??
    } else if files::is_raw(Path::new(&path)) {
        let mut bytes = Vec::new();
        for tag in ["JpgFromRaw", "PreviewImage", "ThumbnailImage"] {
            match service
                .engine
                .one_shot(&[format!("-{tag}"), "-b".into(), path.clone()])
                .await
            {
                Ok((data, _)) if !data.is_empty() => {
                    bytes = data;
                    break;
                }
                _ => {}
            }
        }
        if bytes.is_empty() {
            return Ok(None);
        }
        bytes
    } else {
        #[cfg(target_os = "macos")]
        {
            if !["heic", "heif", "avif"].contains(&ext.as_str()) {
                return Ok(None);
            }
            let temp =
                std::env::temp_dir().join(format!("tagryn-preview-{}.jpg", uuid::Uuid::new_v4()));
            let mut cmd = tokio::process::Command::new("/usr/bin/sips");
            let output = tokio::time::timeout(
                std::time::Duration::from_secs(20),
                cmd.args(["-s", "format", "jpeg", "-Z", "1280", &path, "--out"])
                    .arg(&temp)
                    .kill_on_drop(true)
                    .output(),
            )
            .await
            .map_err(|_| "Native preview timed out")?
            .map_err(|e| e.to_string())?;
            if !output.status.success() {
                let _ = std::fs::remove_file(&temp);
                return Ok(None);
            }
            let bytes = std::fs::read(&temp).map_err(|e| e.to_string())?;
            let _ = std::fs::remove_file(temp);
            bytes
        }
        #[cfg(not(target_os = "macos"))]
        {
            return Ok(None);
        }
    };
    let thumbnail = tokio::task::spawn_blocking(move || {
        let mut reader = ImageReader::new(Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|e| e.to_string())?;
        let mut limits = image::Limits::default();
        limits.max_alloc = Some(256 * 1024 * 1024);
        limits.max_image_width = Some(30000);
        limits.max_image_height = Some(30000);
        reader.limits(limits);
        let mut decoder = reader.into_decoder().map_err(|e| e.to_string())?;
        let orientation = decoder.orientation().map_err(|e| e.to_string())?;
        let mut decoded = image::DynamicImage::from_decoder(decoder).map_err(|e| e.to_string())?;
        decoded.apply_orientation(orientation);
        let decoded = decoded.thumbnail(1280, 960).to_rgb8();
        let mut out = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(decoded)
            .write_to(&mut out, image::ImageFormat::Jpeg)
            .map_err(|e| e.to_string())?;
        Ok::<_, String>(format!(
            "data:image/jpeg;base64,{}",
            STANDARD.encode(out.into_inner())
        ))
    })
    .await
    .map_err(|e| e.to_string())??;
    let mut cache = service.previews.lock().map_err(|e| e.to_string())?;
    if cache.len() >= 64 {
        cache.clear();
    }
    cache.insert(key, thumbnail.clone());
    Ok(Some(thumbnail))
}
