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
    fn list_commits_over(&self, from: &str) -> Result<Vec<String>>;
    fn list_all_commits(&self) -> Result<Vec<String>>;
    fn get_log_for(&self, commit: &str) -> Result<(String, Option<String>)>;
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
        let content = self.run("rev-list", ["--reverse", &spec])?;

        Ok(split_lines(&content))
    }

    fn list_all_commits(&self) -> Result<Vec<String>> {
        let content = self.run("rev-list", ["--reverse", self.branch()])?;

        Ok(split_lines(&content))
    }

    fn get_log_for(&self, commit: &str) -> Result<(String, Option<String>)> {
        let content = self.run("log", ["-n", "1", "--format='%s%n%n%b'", commit])?;
        let parts: Vec<&str> = content.trim().splitn(3, '\n').collect();

        let subject = parts[0].to_string();
        let body = if let Some(b) = parts.get(2) { b } else { "" };
        let body = body.trim();
        let body = if !body.is_empty() {
            Some(body.to_string())
        } else {
            None
        };

        Ok((subject, body))
    }
}

fn split_lines(content: &str) -> Vec<String> {
    if content.is_empty() {
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
            .with_status(0)
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
            .with_status(0)
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
        assert_eq!(
            err.to_string(),
            "'git rev-list' failed: fatal: some problem with index"
        );
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
            .with_status(0)
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
            .with_status(0)
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
            .with_stderr("fatal: some problem with index")
            .with_status(10)
            .register();

        let result = git.list_all_commits();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "'git rev-list' failed: fatal: some problem with index"
        );
    }

    #[test]
    fn get_log_for_pass() {
        let git = GitOps::new("get-log-pass");
        let msg = vec![
            "feat: the new feature",
            "",
            "Added and amazing new feature!\nContributed-by: someone <me@example.com>",
        ];
        let stdout = msg.join("\n");

        let commit = "abcdef1";
        mock("git")
            .with_arg("log")
            .with_arg("-n")
            .with_arg("1")
            .with_arg("--format='%s%n%n%b'")
            .with_arg(commit)
            .with_stdout(stdout)
            .with_status(0)
            .register();
        let result = git.get_log_for(commit);
        assert!(result.is_ok());
        let (subject, body) = result.unwrap();
        assert_eq!(subject, msg[0]);
        assert_eq!(body, Some(msg[2].to_string()));
    }

    #[test]
    fn get_log_no_body() {
        let git = GitOps::new("get-log-subject-only");
        let msg = vec!["feat: the new feature", "", ""];
        let stdout = msg.join("\n");

        let commit = "7654321";
        mock("git")
            .with_arg("log")
            .with_arg("-n")
            .with_arg("1")
            .with_arg("--format='%s%n%n%b'")
            .with_arg(commit)
            .with_stdout(stdout)
            .with_status(0)
            .register();
        let result = git.get_log_for(commit);
        assert!(result.is_ok());
        let (subject, body) = result.unwrap();
        assert_eq!(subject, msg[0]);
        assert_eq!(body, None);
    }

    #[test]
    fn get_log_errored() {
        let git = GitOps::new("get-log-errored");

        let commit = "a0b1c2d";
        mock("git")
            .with_arg("log")
            .with_arg("-n")
            .with_arg("1")
            .with_arg("--format='%s%n%n%b'")
            .with_arg(commit)
            .with_stderr("fatal: unknown revision or path not in the working tree")
            .with_status(128)
            .register();
        let result = git.get_log_for(commit);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(
            err.to_string(),
            "'git log' failed: fatal: unknown revision or path not in the working tree"
        );
    }
}
