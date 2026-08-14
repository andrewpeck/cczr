//! Exit-code behaviour when cczr is used as the tail of a shell pipeline.
//!
//! In a plain pipeline `some_command | cczr`, the pipeline's status is cczr's
//! own status — the upstream command's exit code is not visible to cczr, which
//! is a separate process. What surfaces an upstream failure is the shell's
//! `pipefail` option, and for that to work cczr must NOT mask the upstream code:
//! it has to exit 0 on normal completion so `pipefail` reports the failing
//! side's status.
//!
//! These tests pin both halves of that contract:
//!   1. cczr exits 0 on success (so it never masks an upstream failure).
//!   2. Under `pipefail`, an upstream command's non-zero exit propagates
//!      through `some_command | cczr`.

use std::process::Command;

/// Path to the freshly-built `cczr` binary, provided by Cargo to integration tests.
const CCZR: &str = env!("CARGO_BIN_EXE_cczr");

/// Run a bash pipeline with `pipefail` enabled and return its exit code.
fn pipefail_status(pipeline: &str) -> i32 {
    let status = Command::new("bash")
        .args(["-o", "pipefail", "-c", pipeline])
        .status()
        .expect("failed to spawn bash");
    status
        .code()
        .expect("process terminated by signal, no exit code")
}

#[test]
fn cczr_exits_zero_on_success() {
    // A clean upstream feeding cczr must yield a clean pipeline status,
    // otherwise cczr would itself be the thing masking upstream failures.
    let code = pipefail_status(&format!("printf 'hello world\\n' | {CCZR} > /dev/null"));
    assert_eq!(code, 0, "cczr should exit 0 when the pipeline succeeds");
}

#[test]
fn upstream_failure_propagates_through_cczr() {
    // `false` exits 1; with pipefail the pipeline status must be that failure,
    // not swallowed by cczr on the receiving end.
    let code = pipefail_status(&format!("false | {CCZR} > /dev/null"));
    assert_eq!(code, 1, "upstream failure should propagate, not be masked");
}

#[test]
fn upstream_exit_code_is_preserved_exactly() {
    // The specific non-zero code (not just "non-zero") must survive the pipe.
    let code = pipefail_status(&format!("sh -c 'exit 42' | {CCZR} > /dev/null"));
    assert_eq!(code, 42, "the exact upstream exit code should be preserved");
}
