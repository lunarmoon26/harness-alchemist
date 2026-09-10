use std::{
    ffi::{OsStr, OsString},
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::Duration,
};

use wait_timeout::ChildExt;

#[derive(Default)]
pub struct RunOptions {
    pub cwd: Option<PathBuf>,
    pub env: Vec<(OsString, OsString)>,
    pub timeout: Option<Duration>,
}

pub struct RunResult {
    pub ok: bool,
    pub missing: bool,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

#[cfg(unix)]
fn terminate_process_tree(child: &mut std::process::Child) {
    let group = -(child.id() as i32);
    // SAFETY: kill is called with the process group created immediately before spawn.
    unsafe {
        libc::kill(group, libc::SIGTERM);
    }
    thread::sleep(Duration::from_millis(250));
    if child.try_wait().ok().flatten().is_none() {
        // SAFETY: the saved process-group id remains valid until the child is reaped.
        unsafe {
            libc::kill(group, libc::SIGKILL);
        }
    }
}

#[cfg(windows)]
fn terminate_process_tree(child: &mut std::process::Child) {
    let _ = Command::new("taskkill")
        .args(["/PID", &child.id().to_string(), "/T", "/F"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let _ = child.kill();
}

pub fn run<I, S>(program: &str, args: I, options: RunOptions) -> RunResult
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(cwd) = options.cwd {
        command.current_dir(cwd);
    }
    command.envs(options.env);
    configure_process_group(&mut command);

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return RunResult {
                ok: false,
                missing: error.kind() == std::io::ErrorKind::NotFound,
                stdout: String::new(),
                stderr: error.to_string(),
                timed_out: false,
            };
        }
    };

    let stdout = child.stdout.take().map(|mut stream| {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stream.read_to_end(&mut bytes);
            bytes
        })
    });
    let stderr = child.stderr.take().map(|mut stream| {
        thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stream.read_to_end(&mut bytes);
            bytes
        })
    });

    let timeout = options.timeout.unwrap_or(Duration::from_secs(300));
    let (status, timed_out) = match child.wait_timeout(timeout) {
        Ok(Some(status)) => (Some(status), false),
        Ok(None) => {
            terminate_process_tree(&mut child);
            (child.wait().ok(), true)
        }
        Err(error) => {
            terminate_process_tree(&mut child);
            let _ = child.wait();
            let mut stderr_bytes = stderr
                .and_then(|handle| handle.join().ok())
                .unwrap_or_default();
            stderr_bytes.extend_from_slice(error.to_string().as_bytes());
            return RunResult {
                ok: false,
                missing: false,
                stdout: String::from_utf8_lossy(
                    &stdout
                        .and_then(|handle| handle.join().ok())
                        .unwrap_or_default(),
                )
                .into_owned(),
                stderr: String::from_utf8_lossy(&stderr_bytes).into_owned(),
                timed_out: false,
            };
        }
    };

    let stdout = stdout
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    let mut stderr = stderr
        .and_then(|handle| handle.join().ok())
        .unwrap_or_default();
    if timed_out && stderr.is_empty() {
        stderr.extend_from_slice(
            format!("{program} timed out after {}ms", timeout.as_millis()).as_bytes(),
        );
    }
    let code = status.and_then(|status| status.code());
    RunResult {
        ok: !timed_out && code == Some(0),
        missing: false,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        timed_out,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn terminates_a_timed_out_process_group() {
        let result = run(
            "sh",
            ["-c", "sleep 5"],
            RunOptions {
                timeout: Some(Duration::from_millis(25)),
                ..RunOptions::default()
            },
        );
        assert!(!result.ok);
        assert!(result.timed_out);
    }
}
