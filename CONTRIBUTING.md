# CONTRIBUTING

Thanks for your interesting in contributing! Feel free to create pull-request from your fork.

> [!IMPORTANT]
> By submitting a PR, you agree that the copyright for your changes belongs with the [AUTHORS](AUTHORS).
> 
> Regular contributors will be added to the `AUTHORS` file; we appreciate your understanding that what "regular" means is very subjective at this time.

## PREREQUISITES

The following are needed to develop `changelogger`:

* [Rust](https://rust-lang.org/), version 1.93 or higher
* [Git](https://git-scm.org/), version 2.0 or higher

The following are recommended

* [Lefthook](https://github.com/evilmartians/lefthook), version 2.0 or higher

## DEVCONTAINER

This projects provides a [devcontainer](https://containers.dev) environment. All of the [development](#developing) tasks can be done within the devcontainer. However, the default target will be a linux OS matching your host's architecture, regardless of what your workstation is (e.g., M1 MacOS). The devcontainer is initialized with all the configuration and prerequisites necessary to [cross-compile](https://rust-lang.github.io/rustup/cross-compilation.html) to the following targets:

* `aarch64-apple-darwin`
* `aarch64-unknown-linux-musl`
* `x86_64-apple-darwin`
* `x86_64-unknown-linux-musl`

> [!NOTE]
> Cross-compilation uses the [LLVM linker](https://lld.llvm.org/) `lld`. Although compiled without `glibc`, the produced binary should funcation without issue on just about every Linux distribution for the supported architecture (e.g., `aarch64`).

## DEVELOPING

To build the binary:

```bash
cargo build
```

To build the binary for a specific target (e.g., `aarch64-apple-darwin`):

```bash
cargo build --target aarch64-apple-darwin
```

To compile and run tests:

```bash
cargo test
```

To comple and run tests, generating a coverage report using [cargo-llv-cov](https://github.com/taiki-e/cargo-llvm-cov):

```bash
cargo llvm-cov
```

> [!NOTE]
> All of the report formats that `cargo-llvm-cov` supports can be generated.  Note that if running inside a [devcontainer](#devcontainer) that `--open` will not behave as expected.  The report is still generated on the host workspace and can be viewed through the host's shell and directories.

## BEST PRACTICE

Try to include tests with all changes.  We may request changes to your PR if they're lacking or insufficient.

Coding standards are enforced via CI. To save some roundtrips, consider installing and enabling `lefthook` on your working repository:

```bash
$ lefthook install
```
