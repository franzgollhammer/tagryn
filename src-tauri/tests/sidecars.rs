mod common;
use common::*;
use serde_json::json;
use std::path::Path;
use tagryn_lib::{files, jobs, metadata, model::*};

#[test]
fn sidecar_creation_conflicts_and_contact_fields() {
    run(async {
        let f = Fixture::new();
        let file = f.file("sidecar.jpg");
        let before = files::fingerprint(Path::new(&file.path)).unwrap();
        let mut req = f
            .request(
                &file,
                vec![
                    Edit {
                        field: "title".into(),
                        value: json!("Sidecar title"),
                        mode: "replace".into(),
                    },
                    Edit {
                        field: "email".into(),
                        value: json!("editor@example.test"),
                        mode: "replace".into(),
                    },
                ],
            )
            .await;
        req.force_sidecar = true;
        let plan = f.service.plan(vec![req]).await.unwrap();
        assert!(plan.files[0].error.is_none());
        let result = jobs::execute_file(&f.service.engine, &plan.files[0], "sidecar", |_| Ok(()))
            .await
            .unwrap();
        assert_eq!(files::fingerprint(Path::new(&file.path)).unwrap(), before);
        let doc = f.service.document(&file.id, true).await.unwrap();
        assert!(doc.sidecar.is_some());
        assert_eq!(
            metadata::values(&doc.tags, "XMP-dc:Title", "sidecar"),
            json!("Sidecar title")
        );
        assert_ne!(
            metadata::values(&doc.tags, "XMP-dc:Title", "embedded"),
            metadata::values(&doc.tags, "XMP-dc:Title", "sidecar")
        );
        assert_eq!(
            metadata::values(&doc.tags, "XMP-iptcCore:CreatorWorkEmail", "sidecar"),
            json!("editor@example.test")
        );
        let mut req = f
            .request(
                &file,
                vec![Edit {
                    field: "title".into(),
                    value: json!("later"),
                    mode: "replace".into(),
                }],
            )
            .await;
        req.force_sidecar = true;
        let stale = f.service.plan(vec![req]).await.unwrap();
        jobs::restore(&result).unwrap();
        assert!(
            jobs::execute_file(&f.service.engine, &stale.files[0], "stale", |_| Ok(()))
                .await
                .is_err()
        );
        assert!(!Path::new(doc.sidecar.as_ref().unwrap()).exists());
        f.service.engine.shutdown().await;
    });
}

#[test]
fn shift_preserves_existing_exif_timezone_and_missing_stays_missing() {
    run(async {
        let f = Fixture::new();
        let file = f.file("clock.jpg");
        let mut req = f
            .request(
                &file,
                vec![Edit {
                    field: "dateTaken".into(),
                    value: json!(3600),
                    mode: "shift".into(),
                }],
            )
            .await;
        req.sync_legacy = true;
        let plan = f.service.plan(vec![req]).await.unwrap();
        assert!(plan.files[0]
            .operations
            .iter()
            .any(|o| o.value == json!("2026:09:06 07:42:18+02:00")));
        jobs::execute_file(&f.service.engine, &plan.files[0], "clock", |_| Ok(()))
            .await
            .unwrap();
        assert_eq!(
            tagryn_lib::planner::shift_date("2026:09:06 23:59:00", 120).unwrap(),
            "2026:09:07 00:01:00"
        );
        f.service.engine.shutdown().await;
    });
}
