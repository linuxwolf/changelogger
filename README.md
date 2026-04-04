# CHANGELOGGER

A tool that uses [convential commits](https://conventionalcommits.org) and [semantic versioning](https://semver.org) within a [git](https://git-scm.com/) repository to update a changelog.

## PREREQUISITES

The following are expcted to be present and on the `PATH`:

* `git` (version `2.0` or higher)

## INSTALLING

Tee Bee Dee

## USING

For a project adhering to the [defaults](#defaults), a simple run of `changelogger` will perform its job:

```bash
$ changelogger
```

> [!WARNING]
> If the version file is not found, the command fails

> [!NOTE]
> If the changelog file is not found, it is created.

## OPERATION

It updates the changelog using conventions and information from the local git index:

1. Reads the current version information, from the git index
2. Scans all commits to the default branch from the current version up to the head of the default branch
3. Processes commits matching conventioal commit syntax into a changelog section
4. Determines the next version based on the types of commits
5. Writes the updated version information and changelog to the local working copy

### VERSION CALCULATION

The next version is calculated based on the processed conventional commits.  Any commit that does not conform to the Conventional Commit standard is ignored.

Any conventional commit that does not match a configured or default section (by type and/or scope) is not displayed but may still impact the next version; if any of its footers are displayed in the changelog, it impacts the version.

* The major number is incremented if any commits indicate a `BREAKING CHANGE`
* The minor number is incremented if any commits indicate a `feat`
* The patch is incremented if any commits indicate a `fix` or other displayed section or displayed note

> [!NOTE]
> The `!` flag is an alias for the `BREAKING CHANGE` footer; it will be displayed in the notes for breaking changes instead of its relevant section

## CONFIGURING

The primary method to configure `changelogger` is with a configuration file. By default it will try to load one of the following, which can be overridden with `-c`|`--config`:

* `changelogger.yml` (or `.yaml`)
* `changelogger.toml`
* `changelogger.json`
* `.config/changelogger.yml` (or `.yaml`)
* `.config/changelogger.toml`
* `.config/changelogger.json`

> [!WARNING]
> If more than one of the above exist, which is used is undeterined.
> 
> **No more than one default configuraiton file should exist.**

> [!IMPORTANT]
> If a configuration file is specified on the command-line, the tool fails if that file cannot be read

If a default configuration file is not found, the [defaults](#defaults) are used.

Many (but not all) configuration properties can be also be set on the command line. Any properties set from the command-line override the configuration file.

### Schema

**Root**
* `version-file` _string_ (CLI: `--version-file`) — the file containing version information. The following formats are supported based on the filename:
  * plaintext (`VERSION`, `VERSION.txt`, `version.txt`) — contains just the semantic version
  * JSON (`*.json`) — is an object which contains a `version` field
  * Cargo (`Cargo.toml`) — contains a `package` section with `version` field
* `version-prefix` (CLI: `--version-prefix`) — the prefix to expect when matching git tags, and accommodating in version files (i.e., ignoring/stripping if found)
* `changelog-file` _string_ (CLI: `--changelog-file`) — the file containing the changelog. This file is always assumed to be Markdown, regardless of the filename and its extension
* `default-branch` _string_ (CLI: `--default-branch`) — the name of the default branch in the git repository; used directly for most of the git operations
* `sections` _Section[]_ — A list of `Section` objects, applied in order.
* `include-default-sections` _boolean_ (default: `true`) — whether to also include the default sections, after any explicit sections from above
* `notees` _Note[]_ — A list of `Note` object, applied in order. Notes map to footers in the commit messages, but are applied **before** sections in the changelog!

**Section**
* `type` _string_ — The type of conventional commit it applies to (e.g., `feat`, `bug`, etc).
* `scope` _string_ (_optional_) — The cope of conventioanl commit it applies to; if omitted it applies to all scopes for the matching `type`
* `hidden` _boolean_ (default `false`) — If `true`, the section is not displayed, but may still apply to the overall version calculation (such as if its footer is displayed); this also overrides any default section that would match
* `title` _string_ (_optional_) — The section title to use in the changelog; if not specified and not hidden the title is `type` is pluralized and capitalized

**Note**
* `token` _string_ — the footer token it applies to (e.g., `Acked-by`, `BREAKING CHANGE` etc).
* `title` _string_ (_optional_) — The note title to use in the changelog; if not specified the title is `token` with hypens replaced with spaces, the first word capitalized, all other characters converted to lowercase, and the last word pluralized
* `hidden` _boolean_ (default `false`) — If `true`, the note is not displayed.

> [!IMPORTANT]
> The `BREAKING CHANGE` note is treated special: it cannot be hidden, although its title can be changed. Any footer that matches is always displayed.

### Defaults

The following defaults are applied:

| Setting          | Default value  |
| ---------------- | -------------- |
| `version-file`   | `VERSION`      |
| `changelog-file` | `CHANGELOG.md` |
| `default-branch` | `main`         |
| `version-prefix` | `v`            |

## CONTRIBUTING

See (CONTRIBUTING)[CONTRIBUTING.md] for information on developing and contributing to this project.
