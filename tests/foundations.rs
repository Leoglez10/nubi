use nubi::course::{load, Check, Exercise, NodeKind};
use nubi::{checker, emulator, lab};
use std::path::Path;

#[test]
fn s3_foundations_add_objects_and_prefixes_without_replacing_existing_stops() {
    let content = load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("content")).unwrap();
    assert_eq!(
        content.order(),
        vec!["s3-buckets", "lab-primer-bucket", "s3-objetos", "lab-archivo-s3", "dynamodb-claves", "lab-dynamodb"]
    );
    let lesson = &content.nodes["s3-objetos"];
    assert_eq!(lesson.kind, NodeKind::Lesson);
    assert_eq!(lesson.exercises.len(), 4);
    assert!(matches!(&lesson.exercises[0], Exercise::Choice { options, answer, .. }
        if options[*answer] == "La clave completa: informes/enero.txt"));
    assert!(matches!(&lesson.exercises[1], Exercise::Command { answers, .. }
        if answers.contains(&"aws s3 cp enero.txt s3://archivo-nubi/informes/enero.txt".to_string())));
    assert!(matches!(&lesson.exercises[2], Exercise::Order { steps, .. } if steps.len() == 3));
    assert!(matches!(&lesson.exercises[3], Exercise::Match { pairs, .. } if pairs.len() == 3));

    let lab = &content.nodes["lab-archivo-s3"];
    assert_eq!(lab.kind, NodeKind::Lab);
    assert_eq!(lab.files.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(), vec!["enero.txt", "febrero.txt"]);
    assert!(lab.files.iter().all(|f| !f.content.is_empty()));
    assert_eq!(lab.objectives.iter().map(|o| o.check.clone()).collect::<Vec<_>>(), vec![
        Check::BucketExists { bucket: "archivo-nubi".into() },
        Check::ObjectExists { bucket: "archivo-nubi".into(), key: "informes/enero.txt".into() },
        Check::ObjectExists { bucket: "archivo-nubi".into(), key: "informes/febrero.txt".into() },
    ]);
}

/// Runs the shipped hints through the same parser, runner and checker as the app.
/// No student resources: checks and hints are rebound to a process-specific bucket.
#[test]
#[ignore = "requires the local AWS emulator and AWS CLI"]
fn s3_archive_hints_complete_each_objective_and_reset_for_replay() {
    emulator::point_at_emulator();
    let content = load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("content")).unwrap();
    let mut node = content.nodes.into_iter().find(|(id, _)| id == "lab-archivo-s3").unwrap().1;
    let bucket = format!("labcloud-archive-{}", std::process::id());
    for objective in &mut node.objectives {
        objective.hint = objective.hint.replace("archivo-nubi", &bucket);
        match &mut objective.check {
            Check::BucketExists { bucket: b } | Check::ObjectExists { bucket: b, .. } => *b = bucket.clone(),
            other => panic!("unexpected archive check: {other:?}"),
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join(&bucket);
    let hints = node.objectives.iter().map(|o| o.hint.clone()).collect::<Vec<_>>();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let s3 = checker::client().await;
        checker::reset(&s3, &node.objectives).await;
        for _ in 0..2 {
            let dir = lab::sandbox(&root, &node.id, &node.files).unwrap();
            let mut session = lab::Session::new(3);
            assert_eq!(checker::evaluate(&s3, &node.objectives).await, vec![false; 3]);
            for (i, hint) in hints.iter().enumerate() {
                let args = session.submit(hint, &hints).expect("hint is an allowed aws command");
                let (ok, stdout, stderr) = lab::run_aws(&args, &dir);
                assert!(ok, "hint {hint}: stdout={stdout}, stderr={stderr}");
                if i == 0 {
                    let response: serde_json::Value = serde_json::from_str(&stdout).unwrap();
                    assert_eq!(response["Location"], format!("/{bucket}"));
                    assert!(stderr.is_empty(), "{stderr}");
                } else {
                    let file = &node.files[i - 1];
                    assert!(stdout.contains(&format!("s3://{bucket}/informes/{}", file.name)), "{stdout}");
                    let stored = s3.get_object().bucket(&bucket)
                        .key(format!("informes/{}", file.name)).send().await.unwrap();
                    assert_eq!(stored.body.collect().await.unwrap().into_bytes().as_ref(), file.content.as_bytes());
                }
                session.finish_run(ok, &stdout, &stderr);
                let actual = checker::evaluate(&s3, &node.objectives).await;
                let expected = (0..3).map(|j| j <= i).collect::<Vec<_>>();
                assert_eq!(actual, expected, "only the reached objectives are complete");
                session.update(&actual);
            }
            assert!(session.complete());
            assert_eq!(session.errors, 0);
            assert!(session.mistakes.is_empty());
            checker::reset(&s3, &node.objectives).await;
            assert_eq!(checker::evaluate(&s3, &node.objectives).await, vec![false; 3]);
        }
        checker::reset(&s3, &node.objectives).await;
    });
    std::fs::remove_dir_all(root).unwrap();
}
