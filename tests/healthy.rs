//! Own test binary: it points HOME at an empty folder, which must not race other tests.

use nubi::emulator::{healthy, point_at_emulator};

/// A stopped emulator removes its `ca.pem`; the probe must answer "not ready", not panic.
#[test]
fn a_missing_emulator_certificate_reads_as_not_healthy() {
    let home = std::env::temp_dir().join(format!("nubi-healthy-{}", std::process::id()));
    std::fs::create_dir_all(&home).unwrap();
    // SAFETY: the only test in this binary, before any thread reads the environment.
    unsafe { std::env::set_var("HOME", &home) };
    point_at_emulator();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let ok = rt.block_on(healthy());
    std::fs::remove_dir_all(&home).unwrap();
    assert!(!ok);
}
