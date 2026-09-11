mod common;
use common::*;
use serde_json::json;
use tagryn_lib::model::*;
async fn finished(f: &Fixture, id: &str) -> Job {
    for _ in 0..400 {
        let job = f.service.jobs.lock().unwrap().get(id).cloned().unwrap();
        if !["queued", "running"].contains(&job.status.as_str()) {
            return job;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("Job did not finish")
}
#[test]
fn partial_batch_reports_independent_success_and_failure() {
    run(async {
        let f = Fixture::new();
        let a = f.file("one.jpg");
        let b = f.file("two.jpg");
        let edit = Edit {
            field: "title".into(),
            value: json!("Batch"),
            mode: "replace".into(),
        };
        let requests = vec![
            f.request(&a, vec![edit.clone()]).await,
            f.request(&b, vec![edit]).await,
        ];
        let plan = f.service.plan(requests).await.unwrap();
        std::fs::write(&b.path, b"Changed externally").unwrap();
        let id = f.service.start_job(None, plan.id).unwrap();
        let job = finished(&f, &id).await;
        assert_eq!(job.status, "partial");
        assert_eq!(job.completed, 2);
        assert_eq!(job.errors, 1);
        assert!(job.results[0].backup.is_some());
        assert_eq!(job.results[1].status, "error");
        assert_eq!(std::fs::read(&b.path).unwrap(), b"Changed externally");
        f.service.engine.shutdown().await;
    });
}
#[test]
fn cancellation_of_queued_batch_starts_no_files() {
    run(async {
        let f = Fixture::new();
        let file = f.file("cancel.jpg");
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
        let gate = f.service.write_slot.acquire().await.unwrap();
        let id = f.service.start_job(None, plan.id).unwrap();
        f.service.cancel(&id).unwrap();
        drop(gate);
        let job = finished(&f, &id).await;
        assert_eq!(job.status, "cancelled");
        assert_eq!(job.completed, 0);
        f.service.engine.shutdown().await;
    });
}
