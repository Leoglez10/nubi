//! Course content: the map (units and their stops) and each stop's exercises or lab
//! objectives, loaded from Spanish TOML files so content can grow without touching UI code.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Course {
    pub units: Vec<Unit>,
}

#[derive(Debug, Deserialize)]
pub struct Unit {
    pub id: String,
    pub title: String,
    pub nodes: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    #[serde(rename = "leccion")]
    Lesson,
    #[serde(rename = "lab")]
    Lab,
    #[serde(rename = "desafio")]
    Challenge,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub title: String,
    pub kind: NodeKind,
    #[serde(default)]
    pub exercises: Vec<Exercise>,
    #[serde(default)]
    pub objectives: Vec<Objective>,
    /// Starter files a lab puts in its terminal folder.
    #[serde(default)]
    pub files: Vec<LabFile>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LabFile {
    /// A plain file name: no folders.
    pub name: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Exercise {
    /// Pick one option; `answer` is its index.
    Choice { prompt: String, options: Vec<String>, answer: usize, hint: String },
    /// Type a command; any of `answers` is accepted (whitespace-insensitive).
    Command { prompt: String, answers: Vec<String>, hint: String },
    /// Put `steps` in order; the file lists them in the correct order, the UI shuffles them.
    Order { prompt: String, steps: Vec<String>, hint: String },
    /// Join each left item with its right item; the file lists correct pairs, the UI shuffles.
    Match { prompt: String, pairs: Vec<(String, String)>, hint: String },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Objective {
    pub text: String,
    pub hint: String,
    pub check: Check,
}

/// A rule the checker evaluates against the emulator's state.
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Check {
    BucketExists { bucket: String },
    BucketVersioning { bucket: String, status: String },
    ObjectExists { bucket: String, key: String },
}

#[derive(Debug)]
pub struct Content {
    pub course: Course,
    pub nodes: BTreeMap<String, Node>,
}

impl Content {
    /// Every stop's id, in map order.
    pub fn order(&self) -> Vec<&str> {
        self.course.units.iter().flat_map(|u| u.nodes.iter().map(String::as_str)).collect()
    }
}

/// Loads `<dir>/course.toml` and every `<dir>/lessons/*.toml`, and validates them together.
pub fn load(dir: &Path) -> Result<Content, String> {
    let read = |path: &Path| fs_read(path);
    let course: Course = toml::from_str(&read(&dir.join("course.toml"))?)
        .map_err(|e| format!("course.toml: {e}"))?;

    let mut nodes = BTreeMap::new();
    let lessons = dir.join("lessons");
    let entries = std::fs::read_dir(&lessons).map_err(|e| format!("{}: {e}", lessons.display()))?;
    let mut paths: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for path in paths.into_iter().filter(|p| p.extension().is_some_and(|x| x == "toml")) {
        let file = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        let node: Node = toml::from_str(&read(&path)?).map_err(|e| format!("{file}: {e}"))?;
        validate(&node).map_err(|e| format!("{file}: {e}"))?;
        if path.file_stem().is_none_or(|stem| stem != node.id.as_str()) {
            return Err(format!("{file}: id \"{}\" must match the file name", node.id));
        }
        nodes.insert(node.id.clone(), node);
    }

    let content = Content { course, nodes };
    let mut seen = std::collections::BTreeSet::new();
    for id in content.order() {
        if !content.nodes.contains_key(id) {
            return Err(format!("course.toml: stop \"{id}\" has no file lessons/{id}.toml"));
        }
        if !seen.insert(id) {
            return Err(format!("course.toml: stop \"{id}\" appears twice"));
        }
    }
    Ok(content)
}

fn fs_read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn validate(node: &Node) -> Result<(), String> {
    match node.kind {
        NodeKind::Lab if node.objectives.is_empty() => return Err("a lab needs at least one objective".into()),
        NodeKind::Lesson | NodeKind::Challenge if node.exercises.is_empty() => {
            return Err("a lesson or challenge needs at least one exercise".into());
        }
        _ => {}
    }
    for f in &node.files {
        let plain = !f.name.is_empty() && f.name != "." && f.name != ".." && !f.name.contains(['/', '\\']);
        if !plain {
            return Err(format!("file \"{}\" must be a plain file name", f.name));
        }
    }
    for (i, ex) in node.exercises.iter().enumerate() {
        let n = i + 1;
        match ex {
            Exercise::Choice { options, answer, .. } if *answer >= options.len() => {
                return Err(format!("exercise {n}: answer {answer} is outside its {} options", options.len()));
            }
            Exercise::Choice { options, .. } if options.len() < 2 => return Err(format!("exercise {n}: needs 2+ options")),
            Exercise::Command { answers, .. } if answers.is_empty() => return Err(format!("exercise {n}: needs 1+ answers")),
            Exercise::Order { steps, .. } if steps.len() < 2 => return Err(format!("exercise {n}: needs 2+ steps")),
            Exercise::Match { pairs, .. } if pairs.len() < 2 => return Err(format!("exercise {n}: needs 2+ pairs")),
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn shipped() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("content")
    }

    /// A scratch content folder outside the worktree, removed on drop.
    struct Scratch(PathBuf);
    impl Scratch {
        fn new(name: &str, course: &str, lessons: &[(&str, &str)]) -> Scratch {
            let dir = std::env::temp_dir().join(format!("labcloud-course-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(dir.join("lessons")).unwrap();
            fs::write(dir.join("course.toml"), course).unwrap();
            for (file, body) in lessons {
                fs::write(dir.join("lessons").join(file), body).unwrap();
            }
            Scratch(dir)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    const ONE_UNIT: &str = "[[units]]\nid = \"u\"\ntitle = \"U\"\nnodes = [\"a\"]\n";

    #[test]
    fn the_shipped_course_is_valid() {
        let content = load(&shipped()).unwrap();
        assert_eq!(content.order(), vec!["s3-buckets", "lab-primer-bucket"]);
        assert_eq!(content.nodes["s3-buckets"].kind, NodeKind::Lesson);
        assert_eq!(content.nodes["s3-buckets"].exercises.len(), 5);
        assert_eq!(content.nodes["lab-primer-bucket"].objectives.len(), 3);
        assert_eq!(
            content.nodes["lab-primer-bucket"].objectives[1].check,
            Check::BucketVersioning { bucket: "mi-primer-bucket".into(), status: "Enabled".into() }
        );
    }

    #[test]
    fn a_choice_answer_outside_its_options_is_rejected() {
        let bad = "id = \"a\"\ntitle = \"A\"\nkind = \"leccion\"\n[[exercises]]\ntype = \"choice\"\nprompt = \"?\"\noptions = [\"x\", \"y\"]\nanswer = 2\nhint = \"h\"\n";
        let s = Scratch::new("choice", ONE_UNIT, &[("a.toml", bad)]);
        let err = load(&s.0).unwrap_err();
        assert!(err.contains("a.toml") && err.contains("answer"), "{err}");
    }

    #[test]
    fn a_map_stop_without_a_lesson_file_is_rejected() {
        let s = Scratch::new("missing", ONE_UNIT, &[]);
        let err = load(&s.0).unwrap_err();
        assert!(err.contains("\"a\""), "{err}");
    }

    #[test]
    fn a_lesson_without_exercises_or_a_lab_without_objectives_is_rejected() {
        let empty_lab = "id = \"a\"\ntitle = \"A\"\nkind = \"lab\"\n";
        let s = Scratch::new("empty", ONE_UNIT, &[("a.toml", empty_lab)]);
        assert!(load(&s.0).unwrap_err().contains("a.toml"));
    }

    #[test]
    fn starter_files_cannot_escape_the_lab_folder() {
        let lab = "id = \"a\"\ntitle = \"A\"\nkind = \"lab\"\n[[objectives]]\ntext = \"t\"\nhint = \"h\"\ncheck = { type = \"bucket_exists\", bucket = \"x\" }\n[[files]]\nname = \"../evil.txt\"\ncontent = \"x\"\n";
        let s = Scratch::new("files", ONE_UNIT, &[("a.toml", lab)]);
        assert!(load(&s.0).unwrap_err().contains("plain file name"));
    }

    #[test]
    fn a_file_whose_id_differs_from_its_name_is_rejected() {
        let wrong = "id = \"b\"\ntitle = \"A\"\nkind = \"lab\"\n[[objectives]]\ntext = \"t\"\nhint = \"h\"\ncheck = { type = \"bucket_exists\", bucket = \"x\" }\n";
        let s = Scratch::new("id", ONE_UNIT, &[("a.toml", wrong)]);
        assert!(load(&s.0).unwrap_err().contains("a.toml"));
    }
}
