use crate::HostError;
use lawsynth_plugin_api::PluginKind;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// A process command assembled without shell interpolation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessSpec {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub kind: PluginKind,
}
impl ProcessSpec {
    pub fn validate(&self) -> Result<(), HostError> {
        if self.kind != PluginKind::Process {
            return Err(HostError::Process(
                "only process plugins may be spawned by ProcessSpec".into(),
            ));
        }
        if !self.executable.is_file() {
            return Err(HostError::Process(
                "plugin executable does not exist or is not a regular file".into(),
            ));
        }
        if self.args.iter().any(|arg| arg.contains('\0')) {
            return Err(HostError::Process("arguments cannot contain NUL".into()));
        }
        Ok(())
    }
    pub fn spawn(&self) -> Result<ProcessHandle, HostError> {
        self.validate()?;
        let mut child = Command::new(&self.executable)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| HostError::Process("failed to open child stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| HostError::Process("failed to open child stdout".into()))?;
        Ok(ProcessHandle {
            child,
            stdin,
            stdout,
        })
    }
}
pub struct ProcessHandle {
    child: Child,
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
}
impl ProcessHandle {
    pub fn id(&self) -> u32 {
        self.child.id()
    }
    pub fn try_wait(&mut self) -> Result<Option<std::process::ExitStatus>, HostError> {
        Ok(self.child.try_wait()?)
    }
    /// Wait for normal plugin completion without imposing a host-side signal.
    pub fn wait(&mut self) -> Result<std::process::ExitStatus, HostError> {
        Ok(self.child.wait()?)
    }
    pub fn terminate(&mut self) -> Result<std::process::ExitStatus, HostError> {
        if let Some(status) = self.child.try_wait()? {
            return Ok(status);
        }
        self.child.kill()?;
        Ok(self.child.wait()?)
    }
}
impl Drop for ProcessHandle {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}
