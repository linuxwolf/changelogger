use anyhow::{Context, Result};
use log::{debug, info};

#[cfg(test)]
use mockall::{automock, concretize};

use crate::{
    git::{Git, GitOps},
    settings::Settings,
};

#[cfg_attr(test, automock)]
pub trait App {
    fn get_version(&self) -> Result<String>;
    #[cfg_attr(test, concretize)]
    fn get_version_tag(&self, version: &str) -> Result<Option<String>>;
}

pub struct AppOps<G: Git> {
    settings: Settings,
    git: G,
}

impl<G: Git> AppOps<G> {
    pub fn new(settings: Settings) -> AppOps<GitOps> {
        let branch = &settings.default_branch();
        let git = GitOps::new(branch);

        AppOps::<GitOps> { settings, git }
    }
}

impl<G: Git> App for AppOps<G> {
    fn get_version(&self) -> Result<String> {
        let settings = &self.settings;
        let git = &self.git;

        debug!("read version info from branch {}", git.branch(),);
        let version = git
            .cat_file(settings.version_file())
            .with_context(|| "could not read version information")?
            .trim()
            .to_string();
        info!("current version is {version}");

        Ok(version)
    }

    fn get_version_tag(&self, version: &str) -> Result<Option<String>> {
        let settings = &self.settings;
        let git = &self.git;

        debug!("search for tag matching {version}");
        let prefix = settings.version_prefix();
        let full_version = format!("{prefix}{version}");

        let tags = git.tags()?;
        let result = tags
            .iter()
            .find(|&t| t == version || &full_version == t)
            .map(String::from);

        Ok(result)
    }
}

#[cfg(test)]
mod testing {
    use crate::git::MockGit;

    use super::*;

    fn with_mocks() -> AppOps<MockGit> {
        let settings = Settings::default();
        let mut git = MockGit::new();
        git.expect_branch()
            .return_const(settings.default_branch().to_string());

        AppOps { settings, git }
    }

    #[test]
    fn constructs() {
        let settings = Settings::default();
        let result = AppOps::<GitOps>::new(settings.clone());
        assert_eq!(result.git.branch(), settings.default_branch());
    }

    #[test]
    fn gets_version() {
        let mut app = with_mocks();

        app.git
            .expect_cat_file()
            .withf(|p| p.as_ref() == "VERSION")
            .returning(|_| Ok("2.1.0\n".to_string()));

        let result = app.get_version();
        assert!(result.is_ok());
    }

    #[test]
    fn gets_version_tag_found() {
        let mut app = with_mocks();

        app.git.expect_tags().returning(|| {
            Ok(vec![
                "v0.1.0".to_string(),
                "v0.1.2".to_string(),
                "v0.1.3".to_string(),
                "v1.0.0".to_string(),
                "v1.0.1".to_string(),
                "v1.2.0".to_string(),
                "v1.2.1".to_string(),
                "v2.0.0".to_string(),
                "v2.0.1".to_string(),
                "v2.0.2".to_string(),
                "v2.1.0".to_string(),
            ])
        });
        let version = "2.1.0";
        let result = app.get_version_tag(version);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let tag = result.unwrap();
        assert_eq!(tag, format!("v{version}"));
    }

    #[test]
    fn gets_version_tag_unprefixed() {
        let mut app = with_mocks();

        app.git.expect_tags().returning(|| {
            Ok(vec![
                "0.1.0".to_string(),
                "0.1.2".to_string(),
                "0.1.3".to_string(),
                "1.0.0".to_string(),
                "1.0.1".to_string(),
                "1.2.0".to_string(),
                "1.2.1".to_string(),
                "2.0.0".to_string(),
                "2.0.1".to_string(),
                "2.0.2".to_string(),
                "2.1.0".to_string(),
            ])
        });
        let version = "2.1.0";
        let result = app.get_version_tag(version);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_some());
        let tag = result.unwrap();
        assert_eq!(tag, version);
    }

    #[test]
    fn gets_version_tag_none() {
        let mut app = with_mocks();

        app.git.expect_tags().returning(|| {
            Ok(vec![
                "v0.1.0".to_string(),
                "v0.1.2".to_string(),
                "v0.1.3".to_string(),
                "v1.0.0".to_string(),
                "v1.0.1".to_string(),
                "v1.2.0".to_string(),
                "v1.2.1".to_string(),
                "v2.0.0".to_string(),
            ])
        });
        let version = "2.1.0";
        let result = app.get_version_tag(version);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.is_none());
    }
}
