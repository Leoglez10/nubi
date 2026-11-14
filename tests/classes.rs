use labcloud::course::{load, validate_classes, NodeKind};
use std::path::Path;

#[test]
fn guided_classes_pair_existing_nodes_with_teaching() {
    let content = load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("content")).unwrap();
    let classes = content.classes();
    assert_eq!(classes.len(), 2);
    for (number, lesson, lab) in [(1, "s3-buckets", "lab-primer-bucket"), (2, "s3-objetos", "lab-archivo-s3")] {
        let (node, teaching) = classes[number - 1];
        assert_eq!(node.id, lesson);
        assert_eq!(teaching.number, number);
        assert_eq!(teaching.lab, lab);
        assert_eq!(content.nodes[lab].kind, NodeKind::Lab);
        assert!(!teaching.explanation.is_empty());
        assert!(!teaching.visual.is_empty());
        assert!(!teaching.example.is_empty());
        assert!(!teaching.review.is_empty());
    }
}

#[test]
fn invalid_teaching_and_mappings_are_rejected() {
    let mut content = load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("content")).unwrap();
    let teaching = content.nodes.get_mut("s3-buckets").unwrap().teaching.as_mut().unwrap();
    teaching.lab = "lab-archivo-s3".into();
    assert!(validate_classes(&content).is_err());
    let teaching = content.nodes.get_mut("s3-buckets").unwrap().teaching.as_mut().unwrap();
    teaching.lab = "lab-primer-bucket".into();
    teaching.example = "  ".into();
    assert!(validate_classes(&content).is_err());
}
