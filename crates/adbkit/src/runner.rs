//! Spawning `adb.exe` — the only place in the application that starts a process.
//!
//! Two entry points, because adb has two very different lifecycle shapes:
//!
//! - [`run`] — one-shot commands that produce output (`devices -l`, `getprop`, ...). stdout/stderr are
//!   piped and drained, stdin is null, the child is killed on timeout via `kill_on_drop`.
//! - [`run_server_cmd`] — `start-server` / `kill-server`. **All three stdio handles are null.**
//!
//! The second rule is not cosmetic. A piped client that implicitly spawns the adb daemon lets the daemon
//! inherit the pipe; because the daemon never exits, the read side never sees EOF and the call hangs
//! forever. Upstream hit exactly this and documents it at
//! `Sources/Core/Shell/ShellRunner.swift:58-67` (detached start) and `:123-131` (why the daemon must not
//! inherit handles).

use std::path::Path;
use std::process::{Output, Stdio};
use std::time::Duration;

use tokio::process::Command;

use crate::AdbError;

/// `CREATE_NO_WINDOW` — no console flashes for a GUI app that spawns adb constantly.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// MVP watchdog. Upstream uses 60 s (`Sources/Core/Shell/ShellRunner.swift:33`); nothing in this MVP
/// should legitimately take 20 s, and a shorter bound makes a wedged adb obvious.
pub const TIMEOUT: Duration = Duration::from_secs(20);

/// Run one adb command and capture its output.
///
/// `adb` is the resolved path to `adb.exe` from [`crate::sdk`]. Arguments are passed as argv, never as a
/// shell string, so device-controlled text can never be interpreted by a shell.
pub async fn run(adb: &Path, args: &[&str]) -> Result<Output, AdbError> {
    run_with_timeout(adb, args, TIMEOUT).await
}

/// Same as [`run`] with an explicit timeout (tests use a short one).
pub async fn run_with_timeout(
    adb: &Path,
    args: &[&str],
    timeout: Duration,
) -> Result<Output, AdbError> {
    let mut command = Command::new(adb);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .creation_flags(CREATE_NO_WINDOW);

    let child = command
        .spawn()
        .map_err(|e| AdbError::Spawn(format!("{}: {e}", adb.display())))?;

    // Best effort: if the process already lives in a non-nestable job, assignment fails and the command
    // still has to work. The supervisor is a safety net, not a correctness requirement.
    let _ = crate::winjob::assign(&child);

    match tokio::time::timeout(timeout, child.wait_with_output()).await {
        Ok(Ok(output)) => Ok(output),
        Ok(Err(e)) => Err(AdbError::Io(e.to_string())),
        // Dropping the future drops the `Child`, which kills it (`kill_on_drop`).
        Err(_) => Err(AdbError::Timeout(timeout)),
    }
}

/// `start-server` / `kill-server`. All stdio null — see the module docs for why.
pub async fn run_server_cmd(adb: &Path, arg: &str) -> Result<(), AdbError> {
    run_server_cmd_with_timeout(adb, arg, TIMEOUT).await
}

/// Same as [`run_server_cmd`] with an explicit timeout (tests use a short one).
pub async fn run_server_cmd_with_timeout(
    adb: &Path,
    arg: &str,
    timeout: Duration,
) -> Result<(), AdbError> {
    let mut command = Command::new(adb);
    command
        .arg(arg)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .creation_flags(CREATE_NO_WINDOW);

    let mut child = command
        .spawn()
        .map_err(|e| AdbError::Spawn(format!("{}: {e}", adb.display())))?;
    let _ = crate::winjob::assign(&child);

    match tokio::time::timeout(timeout, child.wait()).await {
        Ok(Ok(_status)) => Ok(()),
        Ok(Err(e)) => Err(AdbError::Io(e.to_string())),
        Err(_) => Err(AdbError::Timeout(timeout)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests never run real adb (`AGENTS.md` §7). `cmd.exe` stands in as a well-behaved child process.
    fn cmd_exe() -> std::path::PathBuf {
        std::path::PathBuf::from(
            std::env::var("COMSPEC").unwrap_or_else(|_| r"C:\Windows\System32\cmd.exe".into()),
        )
    }

    #[tokio::test]
    async fn missing_binary_is_a_spawn_error() {
        let err = run(Path::new(r"C:\does\not\exist\adb.exe"), &["devices"])
            .await
            .unwrap_err();
        assert!(matches!(err, AdbError::Spawn(_)), "got {err:?}");
    }

    #[tokio::test]
    async fn captures_stdout_from_a_child() {
        let out = run(&cmd_exe(), &["/C", "echo hello-adb"]).await.unwrap();
        assert!(out.status.success());
        // Windows children emit CRLF; the parser, not the runner, owns that rule.
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hello-adb");
    }

    #[tokio::test]
    async fn non_zero_exit_is_surfaced_to_the_caller() {
        let out = run(&cmd_exe(), &["/C", "exit 3"]).await.unwrap();
        assert_eq!(out.status.code(), Some(3));
    }

    #[tokio::test]
    async fn a_hung_child_times_out_and_is_killed() {
        let timeout = Duration::from_millis(300);
        let started = std::time::Instant::now();
        let err = run_with_timeout(&cmd_exe(), &["/C", "ping -n 6 127.0.0.1 >nul"], timeout)
            .await
            .unwrap_err();
        assert!(matches!(err, AdbError::Timeout(_)), "got {err:?}");
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "did not return promptly"
        );
    }

    #[tokio::test]
    async fn server_command_tolerates_a_silent_child() {
        // All stdio are null here; the point is that the call returns instead of blocking on EOF.
        run_server_cmd(&cmd_exe(), "/C exit 0").await.unwrap();
    }
}
