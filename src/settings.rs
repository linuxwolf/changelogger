use config::{Config, ConfigError, File, FileFormat, FileSourceFile};
use serde::Deserialize;

use crate::cli::Cli;

#[derive(Clone, Debug, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Settings {
    version_file: Option<String>,
    version_prefix: String,
    changelog_file: String,
    default_branch: Option<String>,
    include_default_sections: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version_file: Some("VERSION".to_string()),
            version_prefix: "v".to_string(),
            changelog_file: "CHANGELOG.md".to_string(),
            default_branch: Some("main".to_string()),
            include_default_sections: true,
        }
    }
}

impl Settings {
    pub fn new(cli: &Cli) -> Result<Settings, ConfigError> {
        let configuring = &cli.configuration;
        let default_files: Vec<File<FileSourceFile, FileFormat>> =
            ["changelogger", ".changelogger", ".config/changelogger"]
                .map(|v| File::with_name(v).required(false))
                .to_vec();
        let config_file = match configuring.config_file.as_ref() {
            Some(config_file) => vec![File::with_name(config_file)],
            None => default_files,
        };
        let settings = Config::builder()
            .add_source(config_file)
            .set_override_option("version-file", configuring.version_file.clone())?
            .set_override_option("version-prefix", configuring.version_prefix.clone())?
            .set_override_option("changelog-file", configuring.changelog_file.clone())?
            .set_override_option("default-branch", configuring.default_branch.clone())?
            .build()?;

        settings.try_deserialize()
    }

    pub fn version_file(&self) -> &str {
        self.version_file.as_deref().unwrap_or("VERSION")
    }

    pub fn default_branch(&self) -> &str {
        self.default_branch.as_deref().unwrap_or("main")
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
    fn only_defaults() {
        let cli = Cli::default();
        let tmp_dir = Temp::new_dir().unwrap();
        let mut cwd = Cwd::mutex().lock().unwrap();
        cwd.set(tmp_dir.as_path()).unwrap();

        let result = Settings::new(&cli);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("VERSION".to_string()));
        assert_eq!(settings.version_prefix, "v".to_string());
        assert_eq!(settings.changelog_file, "CHANGELOG.md".to_string());
        assert_eq!(settings.default_branch, Some("main".to_string()));
        assert_eq!(settings.include_default_sections, true);
    }

    #[test]
    fn from_defaults_files() {
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
        assert_eq!(settings.version_prefix, "ver".to_string());
        assert_eq!(settings.changelog_file, "RELEASE-NOTES.md".to_string());
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
        assert_eq!(settings.version_prefix, "on".to_string());
        assert_eq!(settings.changelog_file, "changes.md".to_string());
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
        assert_eq!(settings.version_prefix, "at".to_string());
        assert_eq!(settings.changelog_file, "releases.md".to_string());
        assert_eq!(settings.default_branch, Some("stable".to_string()));
        assert_eq!(settings.include_default_sections, true);
    }

    #[test]
    fn from_cli_file() {
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
                ..Configuration::default()
            },
            ..Cli::default()
        };
        let result = Settings::new(&cli);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("package.json".to_string()));
        assert_eq!(settings.version_prefix, "ver".to_string());
        assert_eq!(settings.changelog_file, "RELEASE-NOTES.md".to_string());
        assert_eq!(settings.default_branch, Some("master".to_string()));
        assert_eq!(settings.include_default_sections, true);
    }

    #[test]
    fn defaults_with_cli_overrides() {
        let tmp_dir = Temp::new_dir().unwrap();
        let mut cwd = Cwd::mutex().lock().unwrap();
        cwd.set(tmp_dir.as_path()).unwrap();

        let cli = Cli {
            configuration: Configuration {
                version_file: Some("package.json".to_string()),
                ..Configuration::default()
            },
            ..Cli::default()
        };
        let result = Settings::new(&cli);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("package.json".to_string()));
        assert_eq!(settings.version_prefix, "v".to_string());
        assert_eq!(settings.changelog_file, "CHANGELOG.md".to_string());
        assert_eq!(settings.default_branch, Some("main".to_string()));
        assert_eq!(settings.include_default_sections, true);

        let cli = Cli {
            configuration: Configuration {
                version_prefix: Some("ver".to_string()),
                ..Configuration::default()
            },
            ..Cli::default()
        };
        let result = Settings::new(&cli);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("VERSION".to_string()));
        assert_eq!(settings.version_prefix, "ver".to_string());
        assert_eq!(settings.changelog_file, "CHANGELOG.md".to_string());
        assert_eq!(settings.default_branch, Some("main".to_string()));
        assert_eq!(settings.include_default_sections, true);

        let cli = Cli {
            configuration: Configuration {
                changelog_file: Some("RELEASE-NOTES.md".to_string()),
                ..Configuration::default()
            },
            ..Cli::default()
        };
        let result = Settings::new(&cli);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("VERSION".to_string()));
        assert_eq!(settings.version_prefix, "v".to_string());
        assert_eq!(settings.changelog_file, "RELEASE-NOTES.md".to_string());
        assert_eq!(settings.default_branch, Some("main".to_string()));
        assert_eq!(settings.include_default_sections, true);

        let cli = Cli {
            configuration: Configuration {
                default_branch: Some("master".to_string()),
                ..Configuration::default()
            },
            ..Cli::default()
        };
        let result = Settings::new(&cli);
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("VERSION".to_string()));
        assert_eq!(settings.version_prefix, "v".to_string());
        assert_eq!(settings.changelog_file, "CHANGELOG.md".to_string());
        assert_eq!(settings.default_branch, Some("master".to_string()));
        assert_eq!(settings.include_default_sections, true);
    }

    #[test]
    fn file_with_overrides() {
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

        let cli = Cli {
            configuration: Configuration {
                version_file: Some("deno.json".to_string()),
                ..Configuration::default()
            },
            ..Cli::default()
        };
        let result = Settings::new(&cli);
        assert!(result.is_ok());

        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("deno.json".to_string()));
        assert_eq!(settings.version_prefix, "ver".to_string());
        assert_eq!(settings.changelog_file, "RELEASE-NOTES.md".to_string());
        assert_eq!(settings.default_branch, Some("master".to_string()));
        assert_eq!(settings.include_default_sections, true);
    }
}
