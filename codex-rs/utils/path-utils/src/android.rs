use std::ffi::OsStr;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub const TERMUX_PREFIX: &str = "/data/data/com.termux/files/usr";
pub const TERMUX_TMP: &str = "/data/data/com.termux/files/usr/tmp";
pub const TERMUX_BIN_DIR: &str = "/data/data/com.termux/files/usr/bin";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UrlOpener {
    TermuxOpenUrl,
    XdgOpen,
}

impl UrlOpener {
    pub fn command(self) -> &'static str {
        match self {
            Self::TermuxOpenUrl => "termux-open-url",
            Self::XdgOpen => "xdg-open",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BrowserOpenTarget {
    DefaultBrowser,
    Command(UrlOpener),
    PrintUrl,
}

pub fn is_android_termux() -> bool {
    let termux_version = std::env::var_os("TERMUX_VERSION");
    let prefix = std::env::var_os("PREFIX");
    is_android_termux_with_env(cfg!(target_os = "android"), termux_version.as_deref(), prefix.as_deref())
}

pub fn is_android_termux_with_env(
    is_android_target: bool,
    termux_version: Option<&OsStr>,
    prefix: Option<&OsStr>,
) -> bool {
    if !is_android_target {
        return false;
    }

    termux_version.is_some_and(|value| !value.is_empty())
        || prefix.is_some_and(|value| Path::new(value).starts_with(TERMUX_PREFIX))
}

pub fn browser_open_target() -> BrowserOpenTarget {
    browser_open_target_with_env(
        cfg!(target_os = "android"),
        std::env::var_os("TERMUX_VERSION").as_deref(),
        std::env::var_os("PREFIX").as_deref(),
        command_exists(UrlOpener::TermuxOpenUrl.command()),
        command_exists(UrlOpener::XdgOpen.command()),
    )
}

pub fn browser_open_target_with_env(
    is_android_target: bool,
    termux_version: Option<&OsStr>,
    prefix: Option<&OsStr>,
    has_termux_open_url: bool,
    has_xdg_open: bool,
) -> BrowserOpenTarget {
    if !is_android_termux_with_env(is_android_target, termux_version, prefix) {
        return BrowserOpenTarget::DefaultBrowser;
    }

    if has_termux_open_url {
        BrowserOpenTarget::Command(UrlOpener::TermuxOpenUrl)
    } else if has_xdg_open {
        BrowserOpenTarget::Command(UrlOpener::XdgOpen)
    } else {
        BrowserOpenTarget::PrintUrl
    }
}

pub fn run_url_opener(opener: UrlOpener, url: &str) -> io::Result<()> {
    let status = Command::new(opener.command()).arg(url).status()?;
    if status.success() {
        return Ok(());
    }

    Err(io::Error::other(format!(
        "{} exited with status {status}",
        opener.command()
    )))
}

pub fn termux_temp_dir() -> io::Result<PathBuf> {
    let tmpdir = std::env::var_os("TMPDIR");
    let prefix = std::env::var_os("PREFIX");
    let home = dirs::home_dir();
    termux_temp_dir_with_env(
        cfg!(target_os = "android"),
        tmpdir.as_deref(),
        prefix.as_deref(),
        home.as_deref(),
        std::env::temp_dir(),
    )
}

pub fn termux_temp_dir_with_env(
    is_android_target: bool,
    tmpdir: Option<&OsStr>,
    prefix: Option<&OsStr>,
    home: Option<&Path>,
    default_temp_dir: PathBuf,
) -> io::Result<PathBuf> {
    if !is_android_termux_with_env(is_android_target, /*termux_version*/ None, prefix) {
        return Ok(default_temp_dir);
    }

    let mut attempted = Vec::new();
    let fallback_prefix = prefix
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(TERMUX_PREFIX));
    let fallback_candidates = [
        tmpdir.map(PathBuf::from),
        Some(fallback_prefix.join("tmp")),
        Some(PathBuf::from(TERMUX_TMP)),
        home.map(|home| home.join(".cache").join("codex").join("tmp")),
    ];

    for candidate in fallback_candidates.into_iter().flatten() {
        attempted.push(candidate.display().to_string());
        if ensure_writable_dir(&candidate).is_ok() {
            return Ok(candidate);
        }
    }

    Err(io::Error::other(format!(
        "Could not find a writable temporary directory for Android Termux. Tried: {}",
        attempted.join(", ")
    )))
}

fn command_exists(command: &str) -> bool {
    which::which(command).is_ok()
}

fn ensure_writable_dir(path: &Path) -> io::Result<()> {
    std::fs::create_dir_all(path)?;
    let probe_name = format!(
        ".codex-write-test-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let probe_path = path.join(probe_name);
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe_path)?;
    std::fs::remove_file(&probe_path)?;
    Ok(())
}
