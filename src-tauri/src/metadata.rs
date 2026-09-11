use crate::{
    engine::Engine,
    files,
    model::{value_text, Document, FileEntry, Tag},
};
use serde_json::Value;
use std::path::Path;

pub const FIELDS: &[(&str, &str, &str)] = &[
    ("title", "XMP-dc:Title", "text"),
    ("description", "XMP-dc:Description", "text"),
    ("keywords", "XMP-dc:Subject", "list"),
    ("creator", "XMP-dc:Creator", "list"),
    ("copyright", "XMP-dc:Rights", "text"),
    ("rating", "XMP-xmp:Rating", "rating"),
    ("email", "XMP-iptcCore:CreatorWorkEmail", "text"),
    ("phone", "XMP-iptcCore:CreatorWorkTelephone", "text"),
    ("website", "XMP-iptcCore:CreatorWorkURL", "text"),
    ("dateTaken", "XMP-exif:DateTimeOriginal", "date"),
    ("latitude", "XMP-exif:GPSLatitude", "latitude"),
    ("longitude", "XMP-exif:GPSLongitude", "longitude"),
    ("altitude", "XMP-exif:GPSAltitude", "number"),
];
pub fn field(id: &str) -> Option<(&'static str, &'static str)> {
    FIELDS.iter().find(|f| f.0 == id).map(|f| (f.1, f.2))
}
pub fn field_for_tag(tag: &str) -> Option<String> {
    FIELDS.iter().find(|f| f.1 == tag).map(|f| f.0.into())
}
pub fn tag_selector(tag: &Tag) -> String {
    format!("{}:{}", tag.location, tag.name)
}
pub fn values(tags: &[Tag], selector: &str, source: &str) -> Value {
    let found: Vec<_> = tags
        .iter()
        .filter(|t| tag_selector(t) == selector && t.source == source)
        .collect();
    match found.as_slice() {
        [] => Value::Null,
        [tag] => tag.raw.clone(),
        _ => Value::Array(found.iter().map(|t| t.raw.clone()).collect()),
    }
}

pub async fn read_tags(
    engine: &Engine,
    path: &Path,
    source: &str,
) -> Result<(Vec<Tag>, Vec<String>), String> {
    let output = engine
        .execute(
            &[
                "-j",
                "-G:0:1:3:4:5:7",
                "-a",
                "-D",
                "-l",
                "-struct",
                "-api",
                "Struct=2",
                "-api",
                "SaveFormat=1",
                "-charset",
                "filename=UTF8",
            ]
            .map(str::to_owned)
            .into_iter()
            .chain([files::path_string(path)?])
            .collect::<Vec<_>>(),
        )
        .await?;
    let result: Value =
        serde_json::from_str(&output.stdout).map_err(|e| format!("Invalid ExifTool JSON: {e}"))?;
    let object = result
        .as_array()
        .and_then(|a| a.first())
        .and_then(Value::as_object)
        .ok_or("ExifTool returned no document")?;
    let mut tags = Vec::new();
    let mut warnings = Vec::new();
    if !output.warnings.is_empty() {
        warnings.push(output.warnings);
    }
    for (key, data) in object {
        if key == "SourceFile" {
            continue;
        }
        let parts: Vec<_> = key.split(':').collect();
        let name = parts.last().unwrap_or(&key.as_str()).to_string();
        let group = parts.first().unwrap_or(&"Unknown").to_string();
        let location = parts.get(1).unwrap_or(&group.as_str()).to_string();
        let val = data.get("val").unwrap_or(data).clone();
        if name == "Error" {
            return Err(value_text(&val));
        }
        if name == "Warning" {
            warnings.push(value_text(&val));
        }
        let selector = format!("{location}:{name}");
        let field = field_for_tag(&selector);
        tags.push(Tag {
            id: format!("{source}:{key}"),
            key: key.clone(),
            group: group.clone(),
            location,
            name,
            label: data
                .get("desc")
                .and_then(Value::as_str)
                .unwrap_or(key)
                .into(),
            raw: data.get("num").unwrap_or(&val).clone(),
            formatted: val,
            source: source.into(),
            writable: field.is_some(),
            derived: group == "Composite",
            field,
            format: data.get("fmt").map(value_text),
        });
    }
    let mut instances = std::collections::HashMap::new();
    for tag in &tags {
        *instances.entry(tag_selector(tag)).or_insert(0usize) += 1;
    }
    for tag in &mut tags {
        if instances.get(&tag_selector(tag)).copied().unwrap_or(0) > 1 {
            tag.writable = false;
        }
    }
    Ok((tags, warnings))
}

pub async fn read_document(engine: &Engine, file: FileEntry) -> Result<Document, String> {
    let path = Path::new(&file.path);
    let hash_path = path.to_path_buf();
    let fingerprint = tokio::task::spawn_blocking(move || files::fingerprint(&hash_path))
        .await
        .map_err(|e| e.to_string())??;
    let (mut tags, mut warnings) = read_tags(engine, path, "embedded").await?;
    let file_type = tags
        .iter()
        .find(|t| t.name == "FileType")
        .map(|t| value_text(&t.raw))
        .unwrap_or_default();
    let capabilities = files::capabilities(path, &file_type);
    if file.readonly {
        for tag in &mut tags {
            tag.writable = false;
        }
    }
    let sidecar_path = files::existing_sidecar(path);
    let mut sidecar_fingerprint = None;
    if let Some(sidecar) = &sidecar_path {
        sidecar_fingerprint = Some(files::fingerprint(sidecar)?);
        match read_tags(engine, sidecar, "sidecar").await {
            Ok((mut sidecar_tags, sidecar_warnings)) => {
                tags.append(&mut sidecar_tags);
                warnings.extend(sidecar_warnings);
            }
            Err(e) => warnings.push(format!("Sidecar could not be read: {e}")),
        }
        if files::fingerprint(sidecar)?
            != sidecar_fingerprint
                .clone()
                .ok_or("Sidecar fingerprint missing")?
        {
            return Err("Sidecar changed while reading".into());
        }
    }
    if files::fingerprint(path)? != fingerprint {
        return Err("File changed while metadata was read".into());
    }
    if capabilities.write == "readOnly" {
        for t in &mut tags {
            if t.source == "embedded" {
                t.writable = false;
            }
        }
    }
    Ok(Document {
        file,
        fingerprint,
        tags,
        capabilities,
        sidecar: sidecar_path.map(|p| files::path_string(&p)).transpose()?,
        sidecar_fingerprint,
        warnings,
    })
}
