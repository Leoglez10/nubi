//! The lab checker: reads the emulator's state through the AWS SDK and says which
//! objectives are done. It never changes anything; the student does the lab.

use aws_sdk_s3::Client;

use crate::course::{Check, Objective};

/// A fresh client each time picks up the new credentials the emulator writes on every start.
pub async fn client() -> Client {
    Client::new(&aws_config::load_from_env().await)
}

/// Whether one check holds right now.
// ponytail: any SDK error counts as "not done yet"; the UI shows emulator health separately.
pub async fn passes(client: &Client, check: &Check) -> bool {
    match check {
        Check::BucketExists { bucket } => client.head_bucket().bucket(bucket).send().await.is_ok(),
        Check::BucketVersioning { bucket, status } => client
            .get_bucket_versioning()
            .bucket(bucket)
            .send()
            .await
            .is_ok_and(|out| out.status().is_some_and(|s| s.as_str() == status)),
        Check::ObjectExists { bucket, key } => client.head_object().bucket(bucket).key(key).send().await.is_ok(),
    }
}

/// One result per objective, in order.
pub async fn evaluate(client: &Client, objectives: &[Objective]) -> Vec<bool> {
    let mut done = Vec::with_capacity(objectives.len());
    for o in objectives {
        done.push(passes(client, &o.check).await);
    }
    done
}

/// Deletes what a lab's checks refer to (its buckets, every object version included), so a
/// replay starts from scratch. Errors are ignored: whatever is missing is already clean.
// ponytail: one page of versions per bucket (up to 1000); paginate when a lab creates more.
pub async fn reset(client: &Client, objectives: &[Objective]) {
    let mut buckets: Vec<&str> = objectives
        .iter()
        .map(|o| match &o.check {
            Check::BucketExists { bucket } | Check::BucketVersioning { bucket, .. } | Check::ObjectExists { bucket, .. } => bucket.as_str(),
        })
        .collect();
    buckets.sort_unstable();
    buckets.dedup();
    for bucket in buckets {
        if let Ok(list) = client.list_object_versions().bucket(bucket).send().await {
            let versions = list.versions().iter().map(|v| (v.key(), v.version_id()));
            let markers = list.delete_markers().iter().map(|m| (m.key(), m.version_id()));
            for (key, version) in versions.chain(markers) {
                let Some(key) = key else { continue };
                let _ = client.delete_object().bucket(bucket).key(key).set_version_id(version.map(String::from)).send().await;
            }
        }
        let _ = client.delete_bucket().bucket(bucket).send().await;
    }
}
