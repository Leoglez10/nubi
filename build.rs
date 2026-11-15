//! Embeds `content/` into the binary so installed copies do not depend on the source tree.

use std::{env, fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=content");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("content");
    let mut lessons: Vec<_> = fs::read_dir(root.join("lessons"))
        .expect("content/lessons")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    lessons.sort();

    let path = |p: &Path| format!("{:?}", p.display().to_string());
    let mut out = format!("const EMBEDDED_COURSE: &str = include_str!({});\n", path(&root.join("course.toml")));
    out.push_str("const EMBEDDED_LESSONS: &[(&str, &str)] = &[\n");
    for p in &lessons {
        let name = p.file_name().unwrap().to_string_lossy();
        out.push_str(&format!("    ({name:?}, include_str!({})),\n", path(p)));
    }
    out.push_str("];\n");
    fs::write(Path::new(&env::var("OUT_DIR").unwrap()).join("content.rs"), out).unwrap();
}
