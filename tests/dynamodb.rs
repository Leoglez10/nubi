use nubi::{course, lab};

#[test]
fn dynamodb_lab_follows_s3_and_supplies_runnable_hints() {
    let content = course::embedded().unwrap();
    let order = content.order();
    assert_eq!(
        &order[order.len() - 2..],
        ["dynamodb-claves", "lab-dynamodb"]
    );
    let lesson = &content.nodes["dynamodb-claves"];
    assert_eq!(lesson.teaching.as_ref().unwrap().number, 3);
    assert_eq!(lesson.teaching.as_ref().unwrap().lab, "lab-dynamodb");
    let lab_node = &content.nodes["lab-dynamodb"];
    assert_eq!(lab_node.objectives.len(), 3);
    for objective in &lab_node.objectives {
        let lab::Input::Aws(args) = lab::parse(&objective.hint) else {
            panic!("invalid hint")
        };
        assert_eq!(args[0], "dynamodb");
        for name in args.iter().filter_map(|arg| arg.strip_prefix("file://")) {
            let file = lab_node.files.iter().find(|f| f.name == name).unwrap();
            assert!(serde_json::from_str::<serde_json::Value>(&file.content).is_ok());
        }
    }
}

#[test]
fn dynamodb_attempt_checks_real_responses_and_rejects_other_tables() {
    use nubi::dynamo_lab::Attempt;
    use serde_json::json;
    use std::cell::Cell;

    let content = course::embedded().unwrap();
    let objectives = &content.nodes["lab-dynamodb"].objectives;
    let attempt = Attempt::new();
    assert_ne!(attempt.table, Attempt::new().table);
    let hints = attempt.hints(objectives);
    let args = |n: usize| match lab::parse(&hints[n]) {
        lab::Input::Aws(args) => args,
        _ => panic!("invalid hint"),
    };
    let call = |_: &[String]| panic!("rejected commands must not call the CLI");
    let mut outside = args(1);
    outside[3] = "another-table".into();
    let rejected = attempt.execute_with(&outside, objectives, call);
    assert!(!rejected.result.0);
    assert!(rejected.checks.is_none());
    let mut delete = args(0);
    delete[1] = "delete-table".into();
    assert!(!attempt.execute_with(&delete, objectives, call).result.0);

    let item = Cell::new(false);
    let fake = |argv: &[String]| {
        assert!(
            argv.windows(2)
                .any(|a| a == ["--endpoint-url", "https://127.0.0.1:9297"])
        );
        assert!(
            argv.windows(2)
                .any(|a| a[0] == "--table-name" && a[1] == attempt.table)
        );
        let output = match argv[1].as_str() {
            "create-table" => json!({}),
            "describe-table" => {
                json!({"Table": {"TableName": attempt.table, "TableStatus":"ACTIVE",
                "KeySchema":[{"AttributeName":"id","KeyType":"HASH"}],
                "AttributeDefinitions":[{"AttributeName":"id","AttributeType":"S"}]}})
            }
            "put-item" => {
                item.set(true);
                json!({})
            }
            "get-item" if item.get() => {
                json!({"Item":{"id":{"S":"1"},"message":{"S":"Hola Nubi"}}})
            }
            "get-item" => json!({}),
            op => panic!("unexpected operation {op}"),
        };
        (true, output.to_string(), String::new())
    };
    assert_eq!(
        attempt.execute_with(&args(0), objectives, fake).checks,
        Some(vec![true, false, false])
    );
    assert_eq!(
        attempt.execute_with(&args(1), objectives, fake).checks,
        Some(vec![true, true, false])
    );
    let outcome = attempt.execute_with(&args(2), objectives, fake);
    assert!(outcome.read);
    assert_eq!(outcome.checks, Some(vec![true, true, true]));
}
