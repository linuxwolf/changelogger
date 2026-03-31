use config::{Config, ConfigError};
use serde::Deserialize;

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
    pub fn new() -> Result<Settings, ConfigError> {
        let builder = Config::builder();

        let s = builder.build()?;

        s.try_deserialize()
    }
}

#[cfg(test)]
mod testing {
    use super::*;

    use current_dir::Cwd;
    use mktemp::Temp;

    #[test]
    fn settings_load_defaults() {
        let tmp_dir = Temp::new_dir().unwrap();
        let mut cwd = Cwd::mutex().lock().unwrap();
        cwd.set(tmp_dir.as_path()).unwrap();

        let result = Settings::new();
        assert!(result.is_ok());
        let settings = result.unwrap();
        assert_eq!(settings.version_file, Some("VERSION".to_string()));
        assert_eq!(settings.version_prefix, Some("v".to_string()));
        assert_eq!(settings.changelog_file, Some("CHANGELOG.md".to_string()));
        assert_eq!(settings.default_branch, Some("main".to_string()));
        assert_eq!(settings.include_default_sections, true);
    }
}
