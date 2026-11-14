//! Needs Docker running: `cargo test -- --include-ignored`.

use labcloud::emulator::{CONTAINER, healthy, point_at_emulator, start};

#[test]
#[ignore]
fn starting_twice_keeps_one_healthy_emulator() {
    point_at_emulator();
    start().unwrap();
    start().unwrap();
    let ps = std::process::Command::new("docker").args(["ps", "-q", "-f", &format!("name=^{CONTAINER}$")]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&ps.stdout).lines().count(), 1, "exactly one container");
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    assert!(rt.block_on(healthy()));
}
