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
    use std::io::Write;
    use std::process::{Command, Stdio};

    let quiet = |args: &[&str]| Command::new("docker").args(args).stdout(Stdio::piped()).stderr(Stdio::piped()).output();
    let image_ready = quiet(&["image", "inspect", IMAGE]).is_ok_and(|o| o.status.success());
    if !image_ready {
        // The Dockerfile only ADDs from a URL, so it builds without a context folder.
        let mut build = Command::new("docker")
            .args(["build", "--build-arg", &format!("ARCH={}", arch()), "-t", IMAGE, "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("docker: {e}"))?;
        build.stdin.take().expect("piped stdin").write_all(DOCKERFILE.as_bytes()).map_err(|e| e.to_string())?;
        let out = build.wait_with_output().map_err(|e| e.to_string())?;
        if !out.status.success() {
            return Err(last_line(&out.stderr));
        }
    }
    let running = quiet(&["ps", "-q", "-f", &format!("name=^{CONTAINER}$")]).is_ok_and(|o| !o.stdout.is_empty());
    if running {
        return Ok(());
    }
    let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    std::fs::create_dir_all(format!("{home}/.aws/emulator")).map_err(|e| e.to_string())?;
    let out = Command::new("docker").args(run_args(&home)).output().map_err(|e| format!("docker: {e}"))?;
    if out.status.success() { Ok(()) } else { Err(last_line(&out.stderr)) }
}

fn last_line(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("error").to_string()
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
