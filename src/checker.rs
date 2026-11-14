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
