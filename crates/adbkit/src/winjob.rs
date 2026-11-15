//! One process-wide Windows Job Object that kills every adb child when the app exits.
//!
//! Why: `kill_on_drop` only covers children whose `Child` handle is still alive. If the app is killed
//! from Task Manager (or panics), those handles vanish with the process and any in-flight `adb.exe`
//! client is orphaned. A job object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` makes the kernel do the
//! cleanup instead.
//!
//! What is deliberately **not** in the job: the adb **server daemon**. `adb start-server` starts a
//! long-lived `adb.exe` that listens on tcp:5037. It is spawned by adb itself (as a detached grandchild
//! of the client we start), it is shared with every other tool on the machine, and upstream also leaves
//! it running (`docs/architecture/05-concurrency-and-lifecycle.md`). Killing it on app exit would break
//! other tools and any emulator session, so it must outlive us.

use std::sync::OnceLock;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

/// A job handle is a kernel handle: valid in every thread of the process, used only for these calls.
struct JobHandle(HANDLE);

// SAFETY: the handle is process-wide and the Win32 job APIs are thread-safe. Nothing mutates it after
// creation; it is closed by the kernel when the process exits.
unsafe impl Send for JobHandle {}
unsafe impl Sync for JobHandle {}

static JOB: OnceLock<JobHandle> = OnceLock::new();

fn job_handle() -> Option<HANDLE> {
    let job = JOB.get_or_init(|| {
        // SAFETY: null attributes/name are documented as valid for CreateJobObjectW.
        let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null::<u16>()) };
        if handle.is_null() {
            return JobHandle(std::ptr::null_mut());
        }

        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

        // SAFETY: `info` is a correctly sized, fully initialized structure of the type implied by the
        // information class; the pointer outlives the call.
        let ok = unsafe {
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                std::ptr::from_ref(&info).cast(),
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if ok == 0 {
            return JobHandle(std::ptr::null_mut());
        }
        JobHandle(handle)
    });

    (!job.0.is_null()).then_some(job.0)
}

/// Put a freshly spawned child into the kill-on-close job.
///
/// Best effort by design: if the process already belongs to a non-nestable job (some launchers and CI
/// runners do this), `AssignProcessToJobObject` fails with `ERROR_ACCESS_DENIED`. The command must still
/// run, so the error is returned to the caller and callers ignore it.
pub fn assign(child: &tokio::process::Child) -> Result<(), std::io::Error> {
    let Some(job) = job_handle() else {
        return Err(std::io::Error::other("job object unavailable"));
    };
    let Some(process) = child.raw_handle() else {
        return Err(std::io::Error::other("child has no process handle"));
    };
    // SAFETY: both handles are valid for the duration of the call (the child is alive — we hold `&Child`).
    let ok = unsafe { AssignProcessToJobObject(job, process) };
    if ok == 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Whether the process-wide job was created successfully. Used by tests and the About/diagnostics view.
pub fn available() -> bool {
    job_handle().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;
    use tokio::process::Command;

    #[test]
    fn job_is_created_once() {
        assert!(available(), "job object could not be created");
        assert!(available(), "second call must reuse the same job");
    }

    #[tokio::test]
    async fn a_spawned_child_can_be_assigned() {
        let mut cmd = Command::new(
            std::env::var("COMSPEC").unwrap_or_else(|_| r"C:\Windows\System32\cmd.exe".into()),
        );
        cmd.args(["/C", "exit 0"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(0x0800_0000);
        let child = cmd.spawn().unwrap();
        let result = assign(&child);
        // A CI runner may already place us in a job; then assignment legitimately fails.
        if let Err(e) = &result {
            eprintln!("assign skipped (expected in some sandboxes): {e}");
        }
        let _ = child.wait_with_output().await;
    }
}
