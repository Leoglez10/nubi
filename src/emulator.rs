//! Where the local AWS Emulator lives and how this process talks to it.

use std::path::PathBuf;

pub const PORT: u16 = 9297;
pub const IMAGE: &str = "labcloud/aws-emulator";
pub const CONTAINER: &str = "labcloud-emulator";
const DOCKERFILE: &str = include_str!("../emulator/Dockerfile");

/// The emulator package's architecture for this machine.
pub fn arch() -> &'static str {
    if cfg!(target_arch = "aarch64") { "aarch64" } else { "x86_64" }
}

/// `docker run` arguments. 127.0.0.1 only: the emulator is never exposed to the network.
pub fn run_args(home: &str) -> Vec<String> {
    let shared = format!("{home}/.aws/emulator");
    let port = PORT.to_string();
    [
        "run", "-d", "--rm", "--name", CONTAINER, "--memory=2g", "--memory-swap=2g",
        "-p", &format!("127.0.0.1:{port}:{port}"),
        "-e", &format!("HOME={home}"),
        "-v", &format!("{shared}:{shared}"),
        IMAGE, "-p", &port, "--telemetry", "off",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Builds the image (once) and starts the container if it is not running. Waiting for it to
/// become healthy is the caller's job.
pub fn start() -> Result<(), String> {
    start_with_progress(&|_| {})
}

pub fn start_with_progress(feed: crate::setup::Feed<'_>) -> Result<(), String> {
    use crate::setup::{stage, stream};
    use std::process::{Command, Stdio};

    stage(feed, "Comprobando si ya existe la imagen del emulador");
    // Do not print the full image configuration.
    let image_ready = Command::new("docker").args(["image", "inspect", IMAGE])
        .stdout(Stdio::null()).stderr(Stdio::null()).status().is_ok_and(|s| s.success());
    if image_ready {
        stage(feed, "La imagen del emulador ya está disponible");
    } else {
        stage(feed, "Construyendo la imagen del emulador");
        // The Dockerfile only ADDs from a URL, so it builds without a context folder.
        stream(Command::new("docker")
            .args(["build", "--progress=plain", "--build-arg", &format!("ARCH={}", arch()), "-t", IMAGE, "-"]),
            Some(DOCKERFILE.as_bytes()), feed).map_err(|e| format!("docker: {e}"))?.check()?;
    }
    stage(feed, "Comprobando si el contenedor ya está en marcha");
    let out = stream(Command::new("docker").args(["ps", "-q", "-f", &format!("name=^{CONTAINER}$")]), None, feed)
        .map_err(|e| format!("docker: {e}"))?;
    let running = out.stdout.is_some();
    out.check()?;
    if running {
        stage(feed, "El contenedor ya está en marcha");
        return Ok(());
    }
    stage(feed, "Iniciando el contenedor del emulador");
    let home = home().ok_or_else(|| "No se encontró tu carpeta de usuario (HOME o USERPROFILE).".to_string())?;
    let home = home.to_string_lossy().into_owned();
    std::fs::create_dir_all(format!("{home}/.aws/emulator")).map_err(|e| e.to_string())?;
    stream(Command::new("docker").args(run_args(&home)), None, feed).map_err(|e| format!("docker: {e}"))?.check()
}

/// The emulator answers an authenticated S3 call.
pub async fn healthy() -> bool {
    // A stopped emulator removes its ca.pem; with SSL_CERT_FILE pointing at nothing the SDK
    // has no roots at all (and debug builds panic), so that alone means "not running".
    if !dir().join("ca.pem").is_file() {
        return false;
    }
    let config = aws_config::load_from_env().await;
    aws_sdk_s3::Client::new(&config).list_buckets().send().await.is_ok()
}

/// `~/.aws/emulator/<port>/`: the credentials, `config` and `ca.pem` the emulator writes on
/// every start.
/// The user's home folder: HOME, or USERPROFILE on Windows where GUI apps often lack HOME.
fn home() -> Option<PathBuf> {
    let var = |k| std::env::var_os(k).filter(|v| !v.is_empty()).map(PathBuf::from);
    var("HOME").or_else(|| var("USERPROFILE"))
}

pub fn dir() -> PathBuf {
    let home = home().unwrap_or_default();
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
        // An app opened from Finder gets a minimal PATH without Homebrew; add the usual spots
        // so `docker`, `aws` and `brew` are found.
        let mut path = std::env::var("PATH").unwrap_or_default();
        for extra in ["/opt/homebrew/bin", "/usr/local/bin"] {
            if !path.split(':').any(|p| p == extra) {
                path.push(':');
                path.push_str(extra);
            }
        }
        std::env::set_var("PATH", path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_container_is_local_only_and_without_telemetry() {
        let args = run_args("/Users/ana");
        let joined = args.join(" ");
        assert!(joined.contains("-p 127.0.0.1:9297:9297"), "{joined}");
        assert!(!args.iter().any(|a| a == "9297:9297"), "never bound to every interface");
        assert!(joined.contains("-v /Users/ana/.aws/emulator:/Users/ana/.aws/emulator"), "{joined}");
        assert!(joined.ends_with("labcloud/aws-emulator -p 9297 --telemetry off"), "{joined}");
        assert_eq!(&args[..2], ["run", "-d"]);
    }
}
