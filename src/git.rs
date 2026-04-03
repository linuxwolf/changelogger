use std::{ffi::OsStr, io, str};

use anyhow::{Result, anyhow};
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
            return Err(anyhow!("'git {command}' failed: {msg}"));
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
        let tags = content.split('\n');

        Ok(Vec::from_iter(tags.map(String::from)))
    }
}

#[cfg(test)]
mod testing {
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
            .with_arg("--oneline")
            .with_arg("with-success")
            .with_stdout(stdout)
            .with_status(0)
            .register();

        let result = git.run("log", ["--oneline", "with-success"]);
        assert!(result.is_ok());
        assert!(was_command_executed(&[
            "git",
            "log",
            "--oneline",
            "with-success"
        ]));
        assert_eq!(result.unwrap(), stdout);
    }

    #[test]
    fn runs_cmd_errored() {
        let git = GitOps::new("some-branch");

        mock("git")
            .with_arg("log")
            .with_arg("--oneline")
            .with_arg("with-error")
            .with_status(1)
            .with_stderr("fatal: stopped at some error")
            .register();

        let result = git.run("log", ["--oneline", "with-error"]);
        assert!(result.is_err());
        result.expect_err("'git log' failed: fatal: stopped at some error");
        assert!(was_command_executed(&[
            "git",
            "log",
            "--oneline",
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
}
