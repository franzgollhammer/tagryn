mod common;
use common::*;
use serde_json::json;
use std::path::Path;
use tagryn_lib::{files, jobs, metadata, model::*};

#[test]
fn unicode_multiline_save_preserves_pixels_and_restore_bytes() {
    run(async {
        let f = Fixture::new();
        let file = f.file(if cfg!(windows) {
            "Unicode – 東京 filename.jpg"
        } else {
            "Unicode – 東京\nfilename.jpg"
        });
        let before = files::fingerprint(Path::new(&file.path)).unwrap();
        let pixels = image::open(&file.path).unwrap().to_rgb8();
        let req = f
            .request(
                &file,
                vec![Edit {
                    field: "title".into(),
                    value: json!("Grüße\n$(not-a-command) <script>"),
                    mode: "replace".into(),
                }],
            )
            .await;
        let plan = f.service.plan(vec![req]).await.unwrap();
        let result = jobs::execute_file(&f.service.engine, &plan.files[0], "integrity", |_| Ok(()))
            .await
            .unwrap();
        assert_eq!(pixels, image::open(&file.path).unwrap().to_rgb8());
        assert_eq!(
            files::fingerprint(Path::new(result.backup.as_ref().unwrap()))
                .unwrap()
                .sha256,
            before.sha256
        );
        let doc = f.service.document(&file.id, true).await.unwrap();
        assert_eq!(
            metadata::values(&doc.tags, "XMP-dc:Title", "embedded"),
            json!("Grüße\n$(not-a-command) <script>")
        );
        jobs::restore(&result).unwrap();
        assert_eq!(
            files::fingerprint(Path::new(&file.path)).unwrap().sha256,
            before.sha256
        );
        f.service.engine.shutdown().await;
    });
}

#[test]
fn duplicate_tags_and_numeric_values_are_preserved() {
    run(async {
        let f = Fixture::new();
        let file = f.file("duplicates.jpg");
        let doc = f.service.document(&file.id, true).await.unwrap();
        let gps: Vec<_> = doc
            .tags
            .iter()
            .filter(|t| t.name == "GPSLatitude")
            .collect();
        assert!(gps.len() >= 3);
        assert_eq!(
            gps.iter()
                .map(|t| &t.id)
                .collect::<std::collections::HashSet<_>>()
                .len(),
            gps.len()
        );
        assert!(gps
            .iter()
            .any(|t| t.raw.is_number() && t.formatted.is_string()));
        assert!(doc.tags.iter().any(|t| t.name == "Artist"));
        assert!(doc.tags.iter().any(|t| t.name == "Creator"));
        assert!(doc.tags.iter().any(|t| t.name == "By-line"));
        f.service.engine.shutdown().await;
    });
}

#[test]
fn external_changes_and_tampered_backups_are_rejected() {
    run(async {
        let f = Fixture::new();
        let file = f.file("concurrent.jpg");
        let req = f
            .request(
                &file,
                vec![Edit {
                    field: "rating".into(),
                    value: json!(5),
                    mode: "replace".into(),
                }],
            )
            .await;
        let plan = f.service.plan(vec![req]).await.unwrap();
        f.service
            .engine
            .execute(&[
                "-overwrite_original".into(),
                "-XMP-dc:Title=external".into(),
                file.path.clone(),
            ])
            .await
            .unwrap();
        let modified = files::fingerprint(Path::new(&file.path)).unwrap();
        assert!(
            jobs::execute_file(&f.service.engine, &plan.files[0], "conflict", |_| Ok(()))
                .await
                .is_err()
        );
        assert_eq!(files::fingerprint(Path::new(&file.path)).unwrap(), modified);
        let req = f
            .request(
                &file,
                vec![Edit {
                    field: "rating".into(),
                    value: json!(4),
                    mode: "replace".into(),
                }],
            )
            .await;
        let plan = f.service.plan(vec![req]).await.unwrap();
        let saved = jobs::execute_file(&f.service.engine, &plan.files[0], "save", |_| Ok(()))
            .await
            .unwrap();
        std::fs::write(saved.backup.as_ref().unwrap(), b"tampered").unwrap();
        assert!(jobs::restore(&saved)
            .unwrap_err()
            .contains("Backup changed"));
        f.service.engine.shutdown().await;
    });
}

#[test]
fn corrupt_and_readonly_files_never_get_written() {
    run(async {
        let f = Fixture::new();
        let file = f.file("readonly.jpg");
        let req = f
            .request(
                &file,
                vec![Edit {
                    field: "title".into(),
                    value: json!("changed"),
                    mode: "replace".into(),
                }],
            )
            .await;
        let mut permissions = std::fs::metadata(&file.path).unwrap().permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&file.path, permissions).unwrap();
        let plan = f.service.plan(vec![req]).await.unwrap();
        assert!(plan.files[0].error.is_some());
        let corrupt = f.directory.path().join("corrupt.jpg");
        std::fs::write(&corrupt, b"broken JPEG").unwrap();
        let file = f.service.register(&corrupt).unwrap();
        assert!(f.service.document(&file.id, true).await.is_err());
        assert!(f.service.document("unopened", true).await.is_err());
        f.service.engine.shutdown().await;
    });
}
