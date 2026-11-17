//! Own test binary: it rewrites HOME/USERPROFILE, which must not race other tests.

/// Windows GUI apps usually have USERPROFILE but no HOME: the emulator folder must still resolve.
#[test]
fn the_emulator_folder_falls_back_to_userprofile() {
    let profile = std::env::temp_dir().join("nubi-userprofile");
    // SAFETY: the only test in this binary, before any thread reads the environment.
    unsafe {
        std::env::remove_var("HOME");
        std::env::set_var("USERPROFILE", &profile);
    }
    assert_eq!(nubi::emulator::dir(), profile.join(".aws/emulator").join(nubi::emulator::PORT.to_string()));
}
