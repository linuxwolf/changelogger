use std::{ffi::OsStr, io, str};

use log::{error, warn};
use mockcmd::Command;

use crate::errors::{AppError, AppResult};

pub struct GitOps {
    branch: String,
}

impl GitOps {
    pub fn new<S: AsRef<str>>(branch: S) -> GitOps {
        GitOps {
            branch: branch.as_ref().to_string(),
        }
    }

    pub fn cat_file<S: AsRef<OsStr>>(&self, path: S) -> AppResult<String> {
        let path = format!("{}:{}", self.branch, path.as_ref().display());
        self.run("cat-file", ["--textconv", &path])
    }

    pub fn tags(&self) -> AppResult<Vec<String>> {
        let content = self.run("tag", [])?;
        let tags = content.split('\n');
        Ok(Vec::from_iter(tags.map(String::from)))
    }

    fn run<I, S>(&self, cmd: S, args: I) -> AppResult<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let output = Command::new("git").arg(&cmd).args(args).output()?;

        let stderr = str::from_utf8(&output.stderr).map_err(io::Error::other)?;
        if !output.status.success() {
            error!("{}", stderr);
            let command = format!("git {}", cmd.as_ref().display());
            let code = output.status.code();
            let code = code.unwrap_or(-1);
            return Err(AppError::CmdFailed { command, code });
        } else if !stderr.is_empty() {
            warn!("{}", stderr);
        }

        let stdout = str::from_utf8(&output.stdout)
            .map_err(io::Error::other)?
            .to_string();
        Ok(stdout)
    }
}

#[cfg(test)]
mod testing {
    use mockcmd::{mock, was_command_executed};

    use crate::{errors::AppError, git::GitOps};

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
        assert!(matches!(result, Err(AppError::CmdFailed { command, .. }) if command == "git log"));
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
