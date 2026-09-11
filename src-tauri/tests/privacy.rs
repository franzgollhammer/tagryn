mod common;
use common::*;
use std::path::Path;
use tagryn_lib::{files, jobs};

#[test]
fn location_cleanup_rereads_and_preserves_orientation_pixels_and_icc() {
    run(async {
        let f = Fixture::new();
        let file = f.file("private.jpg");
        let thumbnail = f.file("thumbnail.jpg");
        f.service
            .engine
            .execute(&[
                "-overwrite_original".into(),
                "-Orientation#=6".into(),
                format!("-ThumbnailImage<={}", thumbnail.path),
                file.path.clone(),
            ])
            .await
            .unwrap();
        let before = f.service.document(&file.id, true).await.unwrap();
        assert!(
            before.tags.iter().any(|tag| tag.group == "ICC_Profile"),
            "Fixture must include a real ICC profile"
        );
        let pixels = image::open(&file.path).unwrap().to_rgb8();
        let mut req = f.request(&file, vec![]).await;
        req.privacy = Some("location".into());
        let plan = f.service.plan(vec![req]).await.unwrap();
        assert!(plan.files[0]
            .differences
            .iter()
            .any(|d| d.tag.contains("GPS")));
        let saved = jobs::execute_file(&f.service.engine, &plan.files[0], "privacy", |_| Ok(()))
            .await
            .unwrap();
        let after = f.service.document(&file.id, true).await.unwrap();
        assert!(!after
            .tags
            .iter()
            .any(|t| t.name.contains("GPS") || t.name == "ThumbnailImage"));
        for tag in before
            .tags
            .iter()
            .filter(|t| t.name == "Orientation" || t.group == "ICC_Profile")
        {
            assert!(after
                .tags
                .iter()
                .any(|a| a.key == tag.key && a.raw == tag.raw));
        }
        assert_eq!(pixels, image::open(&file.path).unwrap().to_rgb8());
        assert_eq!(
            files::fingerprint(Path::new(saved.backup.as_ref().unwrap()))
                .unwrap()
                .sha256,
            before.fingerprint.sha256
        );
        f.service.engine.shutdown().await;
    });
}
