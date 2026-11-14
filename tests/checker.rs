//! Needs the emulator running: `scripts/emulator.sh start && cargo test -- --include-ignored`.

use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::types::{BucketVersioningStatus, VersioningConfiguration};
use labcloud::checker::{client, evaluate, passes};
use labcloud::course::Check;
use labcloud::emulator::point_at_emulator;

#[test]
#[ignore]
fn each_check_follows_the_emulator_state() {
    point_at_emulator();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let s3 = client().await;
        let bucket = format!("labcloud-check-{}", std::process::id());
        let exists = Check::BucketExists { bucket: bucket.clone() };
        let versioned = Check::BucketVersioning { bucket: bucket.clone(), status: "Enabled".into() };
        let object = Check::ObjectExists { bucket: bucket.clone(), key: "hola.txt".into() };

        assert!(!passes(&s3, &exists).await, "nothing created yet");
        assert!(!passes(&s3, &versioned).await);
        assert!(!passes(&s3, &object).await);

        s3.create_bucket().bucket(&bucket).send().await.unwrap();
        assert!(passes(&s3, &exists).await);
        assert!(!passes(&s3, &versioned).await, "versioning is off on a new bucket");

        let on = VersioningConfiguration::builder().status(BucketVersioningStatus::Enabled).build();
        s3.put_bucket_versioning().bucket(&bucket).versioning_configuration(on).send().await.unwrap();
        assert!(passes(&s3, &versioned).await);

        s3.put_object().bucket(&bucket).key("hola.txt").body(ByteStream::from_static(b"hola")).send().await.unwrap();
        assert!(passes(&s3, &object).await);
    });
}

#[test]
#[ignore]
fn the_first_lab_is_checked_objective_by_objective() {
    point_at_emulator();
    let content = labcloud::course::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("content")).unwrap();
    let lab = &content.nodes["lab-primer-bucket"];
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let results = rt.block_on(async { evaluate(&client().await, &lab.objectives).await });
    assert_eq!(results.len(), 3, "one result per objective");
}

#[test]
#[ignore]
fn reset_removes_a_versioned_bucket_with_its_objects() {
    point_at_emulator();
    let content = labcloud::course::load(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("content")).unwrap();
    let lab = &content.nodes["lab-primer-bucket"];
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let s3 = client().await;
        let bucket = "mi-primer-bucket";
        let _ = s3.create_bucket().bucket(bucket).send().await;
        let on = VersioningConfiguration::builder().status(BucketVersioningStatus::Enabled).build();
        s3.put_bucket_versioning().bucket(bucket).versioning_configuration(on).send().await.unwrap();
        for body in [&b"uno"[..], &b"dos"[..]] {
            s3.put_object().bucket(bucket).key("hola.txt").body(ByteStream::from_static(body)).send().await.unwrap();
        }
        assert!(passes(&s3, &Check::BucketExists { bucket: bucket.into() }).await);

        labcloud::checker::reset(&s3, &lab.objectives).await;
        assert_eq!(evaluate(&s3, &lab.objectives).await, vec![false, false, false], "every objective starts undone");
        labcloud::checker::reset(&s3, &lab.objectives).await; // nothing left: still fine
    });
}
