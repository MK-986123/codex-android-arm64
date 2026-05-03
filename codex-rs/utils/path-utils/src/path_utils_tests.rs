#[cfg(unix)]
mod symlinks {
    use super::super::resolve_symlink_write_paths;
    use pretty_assertions::assert_eq;
    use std::os::unix::fs::symlink;

    #[test]
    fn symlink_cycles_fall_back_to_root_write_path() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let a = dir.path().join("a");
        let b = dir.path().join("b");

        symlink(&b, &a)?;
        symlink(&a, &b)?;

        let resolved = resolve_symlink_write_paths(&a)?;

        assert_eq!(resolved.read_path, None);
        assert_eq!(resolved.write_path, a);
        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod wsl {
    use super::super::normalize_for_wsl_with_flag;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[test]
    fn wsl_mnt_drive_paths_lowercase() {
        let normalized =
            normalize_for_wsl_with_flag(PathBuf::from("/mnt/C/Users/Dev"), /*is_wsl*/ true);

        assert_eq!(normalized, PathBuf::from("/mnt/c/users/dev"));
    }

    #[test]
    fn wsl_non_drive_paths_unchanged() {
        let path = PathBuf::from("/mnt/cc/Users/Dev");
        let normalized = normalize_for_wsl_with_flag(path.clone(), /*is_wsl*/ true);

        assert_eq!(normalized, path);
    }

    #[test]
    fn wsl_non_mnt_paths_unchanged() {
        let path = PathBuf::from("/home/Dev");
        let normalized = normalize_for_wsl_with_flag(path.clone(), /*is_wsl*/ true);

        assert_eq!(normalized, path);
    }
}

mod native_workdir {
    use super::super::normalize_for_native_workdir_with_flag;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_verbatim_paths_are_simplified() {
        let path = PathBuf::from(r"\\?\D:\c\x\worktrees\2508\swift-base");
        let normalized = normalize_for_native_workdir_with_flag(path, /*is_windows*/ true);

        assert_eq!(
            normalized,
            PathBuf::from(r"D:\c\x\worktrees\2508\swift-base")
        );
    }

    #[test]
    fn non_windows_paths_are_unchanged() {
        let path = PathBuf::from(r"\\?\D:\c\x\worktrees\2508\swift-base");
        let normalized =
            normalize_for_native_workdir_with_flag(path.clone(), /*is_windows*/ false);

        assert_eq!(normalized, path);
    }
}

mod android_termux {
    use super::super::BrowserOpenTarget;
    use super::super::UrlOpener;
    use super::super::browser_open_target_with_env;
    use super::super::is_android_termux_with_env;
    use super::super::termux_temp_dir_with_env;
    use pretty_assertions::assert_eq;
    use std::ffi::OsStr;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn detects_termux_on_android_from_termux_version() {
        assert!(is_android_termux_with_env(
            /*is_android_target*/ true,
            Some(OsStr::new("0.118.0")),
            None,
        ));
    }

    #[test]
    fn does_not_detect_termux_on_linux() {
        assert!(!is_android_termux_with_env(
            /*is_android_target*/ false,
            Some(OsStr::new("0.118.0")),
            Some(OsStr::new("/data/data/com.termux/files/usr")),
        ));
    }

    #[test]
    fn prefers_termux_open_url_on_android() {
        let target = browser_open_target_with_env(
            /*is_android_target*/ true,
            Some(OsStr::new("0.118.0")),
            Some(OsStr::new("/data/data/com.termux/files/usr")),
            /*has_termux_open_url*/ true,
            /*has_xdg_open*/ true,
        );

        assert_eq!(target, BrowserOpenTarget::Command(UrlOpener::TermuxOpenUrl));
    }

    #[test]
    fn falls_back_to_xdg_open_on_android() {
        let target = browser_open_target_with_env(
            /*is_android_target*/ true,
            Some(OsStr::new("0.118.0")),
            Some(OsStr::new("/data/data/com.termux/files/usr")),
            /*has_termux_open_url*/ false,
            /*has_xdg_open*/ true,
        );

        assert_eq!(target, BrowserOpenTarget::Command(UrlOpener::XdgOpen));
    }

    #[test]
    fn prints_url_when_no_android_opener_exists() {
        let target = browser_open_target_with_env(
            /*is_android_target*/ true,
            Some(OsStr::new("0.118.0")),
            Some(OsStr::new("/data/data/com.termux/files/usr")),
            /*has_termux_open_url*/ false,
            /*has_xdg_open*/ false,
        );

        assert_eq!(target, BrowserOpenTarget::PrintUrl);
    }

    #[test]
    fn keeps_default_browser_behavior_on_linux() {
        let target = browser_open_target_with_env(
            /*is_android_target*/ false,
            None,
            None,
            /*has_termux_open_url*/ false,
            /*has_xdg_open*/ true,
        );

        assert_eq!(target, BrowserOpenTarget::DefaultBrowser);
    }

    #[test]
    fn prefers_tmpdir_when_writable() -> std::io::Result<()> {
        let temp_home = tempfile::tempdir()?;
        let tmpdir = temp_home.path().join("tmpdir");
        let resolved = termux_temp_dir_with_env(
            /*is_android_target*/ true,
            Some(tmpdir.as_os_str()),
            Some(OsStr::new("/data/data/com.termux/files/usr")),
            Some(temp_home.path()),
            PathBuf::from("/tmp/unchanged"),
        )?;

        assert_eq!(resolved, tmpdir);
        Ok(())
    }

    #[test]
    fn falls_back_to_home_cache_when_termux_tmp_is_unusable() -> std::io::Result<()> {
        let temp_home = tempfile::tempdir()?;
        let unwritable_file = temp_home.path().join("tmp-file");
        fs::write(&unwritable_file, "not-a-directory")?;
        let prefix = temp_home.path().join("prefix");
        fs::write(prefix.as_path(), "also-not-a-directory")?;

        let resolved = termux_temp_dir_with_env(
            /*is_android_target*/ true,
            Some(unwritable_file.as_os_str()),
            Some(prefix.as_os_str()),
            Some(temp_home.path()),
            PathBuf::from("/tmp/unchanged"),
        )?;

        assert_eq!(resolved, temp_home.path().join(".cache/codex/tmp"));
        assert!(Path::new(&resolved).is_dir());
        Ok(())
    }

    #[test]
    fn leaves_linux_temp_dir_unchanged() -> std::io::Result<()> {
        let resolved = termux_temp_dir_with_env(
            /*is_android_target*/ false,
            None,
            None,
            None,
            PathBuf::from("/tmp/linux"),
        )?;

        assert_eq!(resolved, PathBuf::from("/tmp/linux"));
        Ok(())
    }
}

mod path_comparison {
    use super::super::paths_match_after_normalization;
    use std::path::PathBuf;

    #[test]
    fn matches_identical_existing_paths() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;

        assert!(paths_match_after_normalization(dir.path(), dir.path()));
        Ok(())
    }

    #[test]
    fn falls_back_to_raw_equality_when_paths_cannot_be_normalized() {
        assert!(paths_match_after_normalization(
            PathBuf::from("missing"),
            PathBuf::from("missing"),
        ));
        assert!(!paths_match_after_normalization(
            PathBuf::from("missing-a"),
            PathBuf::from("missing-b"),
        ));
    }

    #[cfg(windows)]
    #[test]
    fn matches_windows_verbatim_paths() -> std::io::Result<()> {
        let dir = tempfile::tempdir()?;
        let verbatim_dir = PathBuf::from(format!(r"\\?\{}", dir.path().display()));

        assert!(paths_match_after_normalization(verbatim_dir, dir.path()));
        Ok(())
    }
}
