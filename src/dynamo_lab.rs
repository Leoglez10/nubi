//! DynamoDB teaching attempt. Only the three displayed commands are executable here.
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::course::{Check, Objective};
use crate::lab;
use serde_json::Value;

static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone)]
pub struct Attempt {
    pub table: String,
    read: bool,
    pub busy: bool,
}

impl Attempt {
    pub fn new() -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        Self {
            table: format!("nubi-mensajes-{}-{time:x}-{id:x}", std::process::id()),
            read: false,
            busy: false,
        }
    }

    pub fn hints(&self, objectives: &[Objective]) -> Vec<String> {
        objectives
            .iter()
            .map(|o| o.hint.replace("{table}", &self.table))
            .collect()
    }

    pub fn record_read(&mut self) {
        self.read = true;
    }

    pub fn execute(&self, args: &[String], objectives: &[Objective], cwd: &Path) -> Outcome {
        self.execute_with(args, objectives, |args| lab::run_aws(args, cwd))
    }

    pub fn execute_with(
        &self,
        args: &[String],
        objectives: &[Objective],
        call: impl Fn(&[String]) -> (bool, String, String),
    ) -> Outcome {
        let Some(index) = self
            .hints(objectives)
            .iter()
            .position(|hint| lab::parse(hint) == lab::Input::Aws(args.to_vec()))
        else {
            return Outcome {
                result: (
                    false,
                    String::new(),
                    "Usa uno de los comandos de la pista para la tabla de este intento.".into(),
                ),
                checks: None,
                read: false,
            };
        };
        let invoke = |mut argv: Vec<String>| {
            argv.extend([
                "--endpoint-url".into(),
                format!("https://127.0.0.1:{}", crate::emulator::PORT),
                "--no-cli-pager".into(),
                "--no-cli-auto-prompt".into(),
                "--cli-connect-timeout".into(),
                "3".into(),
                "--cli-read-timeout".into(),
                "5".into(),
            ]);
            call(&argv)
        };
        let result = invoke(args.to_vec());
        let read = matches!(objectives[index].check, Check::DynamoRead)
            && result.0
            && item_matches(&result.1);
        let (table_ok, description, _) = invoke(vec![
            "dynamodb".into(),
            "describe-table".into(),
            "--table-name".into(),
            self.table.clone(),
            "--output".into(),
            "json".into(),
        ]);
        let table = table_ok && table_matches(&description, &self.table);
        let item = table && {
            let (ok, output, _) = invoke(vec![
                "dynamodb".into(),
                "get-item".into(),
                "--table-name".into(),
                self.table.clone(),
                "--key".into(),
                r#"{"id":{"S":"1"}}"#.into(),
                "--consistent-read".into(),
                "--output".into(),
                "json".into(),
            ]);
            ok && item_matches(&output)
        };
        let checks = objectives
            .iter()
            .map(|o| match o.check {
                Check::DynamoTable => table,
                Check::DynamoItem => item,
                Check::DynamoRead => item && (self.read || read),
                _ => false,
            })
            .collect();
        Outcome {
            result,
            checks: Some(checks),
            read,
        }
    }
}

impl Default for Attempt {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Outcome {
    pub result: (bool, String, String),
    pub checks: Option<Vec<bool>>,
    pub read: bool,
}

fn table_matches(output: &str, name: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(output) else {
        return false;
    };
    let table = &value["Table"];
    table["TableName"] == name
        && matches!(table["TableStatus"].as_str(), Some("ACTIVE" | "CREATING"))
        && table["KeySchema"] == serde_json::json!([{"AttributeName":"id","KeyType":"HASH"}])
        && table["AttributeDefinitions"]
            .as_array()
            .is_some_and(|attrs| {
                attrs
                    .iter()
                    .any(|a| a["AttributeName"] == "id" && a["AttributeType"] == "S")
            })
}

fn item_matches(output: &str) -> bool {
    let Ok(value) = serde_json::from_str::<Value>(output) else {
        return false;
    };
    value["Item"]["id"]["S"] == "1" && value["Item"]["message"]["S"] == "Hola Nubi"
}
