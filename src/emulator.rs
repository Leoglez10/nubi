//! Where the local AWS Emulator lives and how this process talks to it.

use std::path::PathBuf;

pub const PORT: u16 = 9297;

/// `~/.aws/emulator/<port>/`: the credentials, `config` and `ca.pem` the emulator writes on
/// every start.
pub fn dir() -> PathBuf {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join(".aws/emulator").join(PORT.to_string())
}

/// Points this process — and every `aws` child it spawns — at the emulator.
/// Same variables as `scripts/emulator.sh env`.
///
/// Call it first thing in `main` (or a test), before other threads read the environment.
pub fn point_at_emulator() {
    let dir = dir();
    let ca = dir.join("ca.pem");
    // SAFETY: callers run this before spawning threads that read the environment.
    unsafe {
        for var in ["AWS_ACCESS_KEY_ID", "AWS_SECRET_ACCESS_KEY", "AWS_SESSION_TOKEN", "AWS_PROFILE"] {
            std::env::remove_var(var);
        }
        std::env::set_var("AWS_ENDPOINT_URL", format!("https://127.0.0.1:{PORT}"));
        std::env::set_var("AWS_SHARED_CREDENTIALS_FILE", dir.join("emulator_credentials"));
        std::env::set_var("AWS_CONFIG_FILE", dir.join("config"));
        std::env::set_var("AWS_CA_BUNDLE", &ca);
        std::env::set_var("SSL_CERT_FILE", &ca);
        std::env::set_var("AWS_EC2_METADATA_DISABLED", "true");
        std::env::set_var("AWS_DISABLE_HOST_PREFIX_INJECTION", "true");
        std::env::set_var("AWS_REGION", "us-east-1");
        std::env::set_var("AWS_DEFAULT_REGION", "us-east-1");
        std::env::set_var("AWS_PAGER", "");
    }
}
