use std::ffi::OsString;
use std::process::{Command, ExitStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shell {
    pub program: OsString,
    pub args: Vec<OsString>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status: ExitStatus,
}

impl ExecOutput {
    pub fn combined(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.stdout.len() + self.stderr.len());
        out.extend_from_slice(&self.stdout);
        out.extend_from_slice(&self.stderr);
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecError {
    MissingCommand,
    ShellNotFound,
    SpawnFailed(String),
}

impl std::fmt::Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecError::MissingCommand => write!(f, "missing command"),
            ExecError::ShellNotFound => write!(f, "shell not found"),
            ExecError::SpawnFailed(err) => write!(f, "failed to run command: {err}"),
        }
    }
}

impl std::error::Error for ExecError {}

pub fn default_shell() -> Result<Shell, ExecError> {
    #[cfg(windows)]
    {
        if let Ok(shell) = std::env::var("SHELL") {
            if !shell.trim().is_empty() {
                return Ok(Shell {
                    program: OsString::from(shell),
                    args: vec![OsString::from("-c")],
                });
            }
        }

        if let Ok(comspec) = std::env::var("COMSPEC") {
            if !comspec.trim().is_empty() {
                return Ok(Shell {
                    program: OsString::from(comspec),
                    args: vec![OsString::from("/C")],
                });
            }
        }

        return Ok(Shell {
            program: OsString::from("cmd.exe"),
            args: vec![OsString::from("/C")],
        });
    }

    #[cfg(not(windows))]
    {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        if shell.trim().is_empty() {
            return Err(ExecError::ShellNotFound);
        }
        Ok(Shell {
            program: OsString::from(shell),
            args: vec![OsString::from("-c")],
        })
    }
}

pub fn build_command(command: &[String], exec: bool) -> Result<Command, ExecError> {
    if command.is_empty() {
        return Err(ExecError::MissingCommand);
    }

    if exec {
        let mut cmd = Command::new(&command[0]);
        cmd.args(&command[1..]);
        return Ok(cmd);
    }

    let shell = default_shell()?;
    let command_str = command.join(" ");
    let mut cmd = Command::new(shell.program);
    cmd.args(shell.args);
    cmd.arg(command_str);
    Ok(cmd)
}

pub fn run_command(mut command: Command) -> Result<ExecOutput, ExecError> {
    let output = command
        .output()
        .map_err(|err| ExecError::SpawnFailed(err.to_string()))?;
    Ok(ExecOutput {
        stdout: output.stdout,
        stderr: output.stderr,
        status: output.status,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(())).lock().expect("env lock")
    }

    #[test]
    fn build_exec_command_uses_first_arg_as_program() {
        let cmd = build_command(&["echo".into(), "hi".into()], true).unwrap();
        assert_eq!(cmd.get_program(), "echo");
    }

    #[cfg(windows)]
    #[test]
    fn default_shell_prefers_comspec_when_shell_unset() {
        let _guard = env_lock();
        let prev_shell = std::env::var("SHELL").ok();
        let prev_comspec = std::env::var("COMSPEC").ok();

        unsafe {
            std::env::remove_var("SHELL");
            std::env::set_var("COMSPEC", "cmd.exe");
        }

        let shell = default_shell().unwrap();
        let program = shell.program.to_string_lossy().to_ascii_lowercase();
        assert!(program.ends_with("cmd.exe"));

        match prev_shell {
            Some(val) => unsafe { std::env::set_var("SHELL", val) },
            None => unsafe { std::env::remove_var("SHELL") },
        }
        match prev_comspec {
            Some(val) => unsafe { std::env::set_var("COMSPEC", val) },
            None => unsafe { std::env::remove_var("COMSPEC") },
        }
    }
}
