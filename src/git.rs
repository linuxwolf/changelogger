use std::{ffi::OsStr, io, str};

use anyhow::{Context, Result, anyhow};
use log::{error, warn};
use mockcmd::Command;

#[cfg(test)]
use mockall::{automock, concretize};

#[cfg_attr(test, automock)]
pub trait Git {
    fn branch(&self) -> &str;

    #[cfg_attr(test, concretize)]
    fn cat_file<S: AsRef<OsStr>>(&self, path: S) -> Result<String>;
    fn tags(&self) -> Result<Vec<String>>;
    #[allow(unused)]
    fn list_commits_over(&self, from: &str) -> Result<Vec<String>>;
    #[allow(unused)]
    fn list_all_commits(&self) -> Result<Vec<String>>;
}

pub struct GitOps {
    branch: String,
}

impl GitOps {
    pub fn new<S: AsRef<str>>(branch: S) -> GitOps {
        GitOps {
            branch: branch.as_ref().to_string(),
        }
    }

    fn run<I, S>(&self, cmd: S, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new("git").arg(&cmd).args(args).output()?;

        let stderr = str::from_utf8(&output.stderr).map_err(io::Error::other)?;
        if !output.status.success() {
            error!("{}", stderr);
            let msg = stderr.split('\n').next_back().unwrap_or_default();
            let command = format!("git {}", cmd.as_ref().display());
            return Err(anyhow!("'{command}' failed: {msg}"));
        } else if !stderr.is_empty() {
            warn!("{}", stderr);
        }

        let stdout = str::from_utf8(&output.stdout)?.to_string();
        Ok(stdout)
    }
}

impl Git for GitOps {
    fn branch(&self) -> &str {
        &self.branch
    }

    fn cat_file<S: AsRef<OsStr>>(&self, path: S) -> Result<String> {
        let branch = self.branch();
        let path = path.as_ref().display();
        let spec = format!("{}:{}", branch, path);

        let result = self.run("cat-file", ["--textconv", &spec])?;
        Ok(result)
    }

    fn tags(&self) -> Result<Vec<String>> {
        let content = self.run("tag", [])?;

        Ok(split_lines(&content))
    }

    fn list_commits_over(&self, from: &str) -> Result<Vec<String>> {
        let spec = format!("{from}..{}", self.branch());
        let content = self
            .run("rev-list", ["--reverse", &spec])
            .with_context(|| format!("could not list commits in git index for range ({spec})"))?;

        Ok(split_lines(&content))
    }

    fn list_all_commits(&self) -> Result<Vec<String>> {
        let content = self
            .run("rev-list", ["--reverse", self.branch()])
            .with_context(|| format!("could not list all commits in git index"))?;

        Ok(split_lines(&content))
    }
}

fn split_lines(content: &str) -> Vec<String> {
    if content.len() == 0 {
        Vec::new()
    } else {
        content.trim().split('\n').map(String::from).collect()
    }
}

#[cfg(test)]
mod testing {
    use std::vec;

    use mockcmd::{mock, was_command_executed};

    use super::*;

    #[test]
    fn runs_success() {
        let git = GitOps::new("some-branch");
        let stdout = "a9b0f9e (HEAD -> main) fix(stuf): feature 1 doesn't work
  3583a39 feat(stuff): cool feature 1
  0fd287f chore(main): initialize project";

        mock("git")
            .with_arg("log")
            .with_arg("--format")
            .with_arg("with-success")
            .with_stdout(stdout)
            .with_status(0)
            .register();

        let result = git.run("log", ["--format", "with-success"]);
        assert!(result.is_ok());
        assert!(was_command_executed(&[
            "git",
            "log",
            "--format",
            "with-success"
        ]));
        assert_eq!(result.unwrap(), stdout);
    }

    #[test]
    fn runs_cmd_errored() {
        let git = GitOps::new("some-branch");

        mock("git")
            .with_arg("log")
            .with_arg("--format")
            .with_arg("with-error")
            .with_status(1)
            .with_stderr("fatal: stopped at some error")
            .register();

        let result = git.run("log", ["--format", "with-error"]);
        assert!(result.is_err());
        result.expect_err("'git log' failed: fatal: stopped at some error");
        assert!(was_command_executed(&[
            "git",
            "log",
            "--format",
            "with-error"
        ]));
    }

    #[test]
    fn cat_file_pass() {
        let git = GitOps::new("cat-file-pass");
        let stdout = "file contents example";

        mock("git")
            .with_arg("cat-file")
            .with_arg("--textconv")
            .with_arg("cat-file-pass:README.md")
            .with_stdout(stdout)
            .with_status(0)
            .register();

        let result = git.cat_file("README.md");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), stdout);
        assert!(was_command_executed(&[
            "git",
            "cat-file",
            "--textconv",
            "cat-file-pass:README.md"
        ]));
    }

    #[test]
    fn cat_file_errored() {
        let git = GitOps::new("cat-file-errored");
        let stderr = "fatal: path 'README.md' does not exist in 'cat-file-errored'";

        mock("git")
            .with_arg("cat-file")
            .with_arg("--textconv")
            .with_arg("cat-file-errored:README.md")
            .with_stderr(stderr)
            .with_status(1)
            .register();

        let result = git.cat_file("README.md");
        assert!(result.is_err());
        assert!(was_command_executed(&[
            "git",
            "cat-file",
            "--textconv",
            "cat-file-errored:README.md"
        ]));
    }

    #[test]
    fn tags_pass_multiple() {
        let git = GitOps::new("some-branch");
        let stdout = "v1.0.0\nv1.1.0";

        mock("git")
            .with_arg("tag")
            .with_stdout(stdout)
            .with_status(0)
            .register();

        let result = git.tags();
        assert!(result.is_ok());
        let tags = result.unwrap();
        assert_eq!(tags, vec!["v1.0.0".to_string(), "v1.1.0".to_string()]);
        assert!(was_command_executed(&["git", "tag"]));
    }

    #[test]
    fn list_commits_over_pass() {
        let git = GitOps::new("over-fixed-history");
        let expected = vec![
            "b4a18697c28fe4aa83bf79d03582d27d4db20489".to_string(),
            "69f92f057fc0640603d71b9c6d6224ed60aefe16".to_string(),
            "70095d769bb7f235bc707c1ef7ee10653dd9df61".to_string(),
            "571e35139871f759261bc3d8d74555a4b3aa8616".to_string(),
            "5604a99af83ffac5c8639db7a5c6f13d4c094afc".to_string(),
            "db86881d1d10f1de4eac8dacf5cdace152eaf2c5".to_string(),
        ];
        let stdout = expected.join("\n");

        mock("git")
            .with_arg("rev-list")
            .with_arg("--reverse")
            .with_arg("v0.1.2..over-fixed-history")
            .with_stdout(stdout)
            .register();

        let result = git.list_commits_over("v0.1.2");
        assert!(result.is_ok());
        let commits = result.unwrap();
        assert_eq!(commits, expected);
    }

    #[test]
    fn list_commits_over_empty() {
        let git = GitOps::new("over-empty-history");

        mock("git")
            .with_arg("rev-list")
            .with_arg("--reverse")
            .with_arg("v0.1.2..over-empty-history")
            .register();

        let result = git.list_commits_over("v0.1.2");
        assert!(result.is_ok());
        let commits = result.unwrap();
        assert_eq!(commits.len(), 0);
    }

    #[test]
    fn list_commits_over_errored() {
        let git = GitOps::new("over-errored-history");

        mock("git")
            .with_arg("rev-list")
            .with_arg("--reverse")
            .with_arg("v0.1.2..over-errored-history")
            .with_stderr("fatal: some problem with index")
            .with_status(10)
            .register();

        let result = git.list_commits_over("v0.1.2");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "could not list commits in git index for range (v0.1.2..over-errored-history)")
        
    }

    #[test]
    fn list_all_commits_pass() {
        let git = GitOps::new("all-fixed-history");
        let expected = vec![
            "b4a18697c28fe4aa83bf79d03582d27d4db20489".to_string(),
            "69f92f057fc0640603d71b9c6d6224ed60aefe16".to_string(),
            "70095d769bb7f235bc707c1ef7ee10653dd9df61".to_string(),
            "571e35139871f759261bc3d8d74555a4b3aa8616".to_string(),
            "5604a99af83ffac5c8639db7a5c6f13d4c094afc".to_string(),
            "db86881d1d10f1de4eac8dacf5cdace152eaf2c5".to_string(),
        ];
        let stdout = expected.join("\n");

        mock("git")
            .with_arg("rev-list")
            .with_arg("--reverse")
            .with_arg("all-fixed-history")
            .with_stdout(stdout)
            .register();

        let result = git.list_all_commits();
        assert!(result.is_ok());
        let commits = result.unwrap();
        assert_eq!(commits, expected);
    }

    #[test]
    fn list_all_commits_empty() {
        let git = GitOps::new("all-empty-history");

        mock("git")
            .with_arg("rev-list")
            .with_arg("--reverse")
            .with_arg("all-empty-history")
            .register();

        let result = git.list_all_commits();
        assert!(result.is_ok());
        let commits = result.unwrap();
        assert_eq!(commits.len(), 0);
    }

    #[test]
    fn list_all_commits_errored() {
        let git = GitOps::new("all-errored-history");

        mock("git")
            .with_arg("rev-list")
            .with_arg("--reverse")
            .with_arg("all-errored-history")
            .with_status(10)
            .register();

        let result = git.list_all_commits();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "could not list all commits in git index");
    }
}
