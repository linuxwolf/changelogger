use config::{Config, ConfigError, File, FileFormat, FileSourceFile};
use serde::Deserialize;

use crate::cli::Cli;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Settings {
    version_file: Option<String>,
    version_prefix: Option<String>,
    changelog_file: Option<String>,
    default_branch: Option<String>,
    include_default_sections: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version_file: Some("VERSION".to_string()),
            version_prefix: Some("v".to_string()),
            changelog_file: Some("CHANGELOG.md".to_string()),
            default_branch: Some("main".to_string()),
            include_default_sections: true,
        }
    }
}

impl Settings {
    pub fn new(cli: &Cli) -> Result<Settings, ConfigError> {
        let default_files: Vec<File<FileSourceFile, FileFormat>> =
            ["changelogger", ".changelogger", ".config/changelogger"]
                .map(|v| File::with_name(v).required(false))
                .to_vec();
        let config_file = match cli.configuration.config_file.as_ref() {
            Some(config_file) => vec![
                File::with_name(config_file),
            ],
            None => default_files,
        };
        let builder = Config::builder();
        let builder = builder.add_source(config_file);

        let s = builder.build()?;

        s.try_deserialize()
    }
}

#[cfg(test)]
mod testing {
    use std::fs;

    use current_dir::Cwd;
    use mktemp::Temp;

    use crate::cli::Configuration;

    use super::*;

    #[test]
    fn settings_load_defaults() {
        let cli = Cli::default();
        let tmp_dir = Temp::new_dir().unwrap();
        let mut cwd = Cwd::mutex().lock().unwrap();
        cwd.set(tmp_dir.as_path()).unwrap();

        let result = Settings::new(&cli);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("VERSION".to_string()));
        assert_eq!(settings.version_prefix, Some("v".to_string()));
        assert_eq!(settings.changelog_file, Some("CHANGELOG.md".to_string()));
        assert_eq!(settings.default_branch, Some("main".to_string()));
        assert_eq!(settings.include_default_sections, true);
    }

    #[test]
    fn settings_load_defaults_file() {
        let cli = Cli::default();
        let mut cwd = Cwd::mutex().lock().unwrap();
        let tmp_dir = Temp::new_dir().unwrap();
        cwd.set(tmp_dir.as_path()).unwrap();
        fs::write(
            ".changelogger.yaml",
            b"
version-file: package.json
version-prefix: ver
changelog-file: RELEASE-NOTES.md
default-branch: master
",
        )
        .unwrap();

        let result = Settings::new(&cli);
        assert!(result.is_ok());

        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("package.json".to_string()));
        assert_eq!(settings.version_prefix, Some("ver".to_string()));
        assert_eq!(
            settings.changelog_file,
            Some("RELEASE-NOTES.md".to_string())
        );
        assert_eq!(settings.default_branch, Some("master".to_string()));
        assert_eq!(settings.include_default_sections, true);

        let tmp_dir = Temp::new_dir().unwrap();
        cwd.set(tmp_dir.as_path()).unwrap();
        fs::write(
            "changelogger.yaml",
            b"
version-file: deno.json
version-prefix: on
changelog-file: changes.md
default-branch: primary
",
        )
        .unwrap();

        let result = Settings::new(&cli);
        assert!(result.is_ok());

        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("deno.json".to_string()));
        assert_eq!(settings.version_prefix, Some("on".to_string()));
        assert_eq!(settings.changelog_file, Some("changes.md".to_string()));
        assert_eq!(settings.default_branch, Some("primary".to_string()));
        assert_eq!(settings.include_default_sections, true);

        let tmp_dir = Temp::new_dir().unwrap();
        let config_dir = tmp_dir.join(".config");
        fs::create_dir_all(config_dir.clone()).unwrap();
        cwd.set(config_dir.as_path()).unwrap();
        fs::write(
            "changelogger.yaml",
            b"
version-file: Cargo.toml
version-prefix: at
changelog-file: releases.md
default-branch: stable
",
        )
        .unwrap();

        let result = Settings::new(&cli);
        assert!(result.is_ok());

        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("Cargo.toml".to_string()));
        assert_eq!(settings.version_prefix, Some("at".to_string()));
        assert_eq!(settings.changelog_file, Some("releases.md".to_string()));
        assert_eq!(settings.default_branch, Some("stable".to_string()));
        assert_eq!(settings.include_default_sections, true);
    }

    #[test]
    fn settings_from_explicit_file() {
        let tmp_dir = Temp::new_dir().unwrap();
        let mut cwd = Cwd::mutex().lock().unwrap();
        cwd.set(tmp_dir.as_path()).unwrap();
        let config_file = tmp_dir.join("release-note-config.yaml");
        fs::write(
            &config_file,
            b"
version-file: package.json
version-prefix: ver
changelog-file: RELEASE-NOTES.md
default-branch: master
",
        )
        .unwrap();

        let cli = Cli {
            configuration: Configuration {
                config_file: config_file.to_str().map(String::from),
            },
            ..Cli::default()
        };
        let result = Settings::new(&cli);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("package.json".to_string()));
        assert_eq!(settings.version_prefix, Some("ver".to_string()));
        assert_eq!(settings.changelog_file, Some("RELEASE-NOTES.md".to_string()));
        assert_eq!(settings.default_branch, Some("master".to_string()));
        assert_eq!(settings.include_default_sections, true);
    }
}
