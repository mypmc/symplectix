use std::env;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command as StdCommand, ExitStatus, Stdio};
use std::time::Duration;

use clap::Parser;
use tokio::process;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time;

// pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Parser)]
pub struct Command {
    /// Redirect the child process stdout.
    #[arg(long, value_name = "PATH")]
    stdout: Option<PathBuf>,

    /// Redirect the child process stderr.
    #[arg(long, value_name = "PATH")]
    stderr: Option<PathBuf>,

    /// Kill the spawned child process after the specified duration.
    #[arg(long, value_name = "DURATION", value_parser = humantime::parse_duration)]
    timeout: Option<Duration>,

    /// Environment variables visible to the spawned process.
    #[arg(long = "env", value_name = "KEY")]
    envs: Vec<String>,

    /// The entrypoint of the child process.
    #[arg(last = true)]
    argv: Vec<String>,
}

#[derive(Debug)]
pub struct Process {
    child: process::Child,
    id: u32,
    timeout: Option<Duration>,
}

#[tracing::instrument(skip(process))]
pub async fn wait_and_stop(mut process: Process) -> io::Result<(ExitStatus, bool)> {
    let mut interrupt = signal(SignalKind::interrupt())?;
    let mut terminate = signal(SignalKind::terminate())?;

    let timedout = tokio::select! {
        biased;
        _ = interrupt.recv() => { false },
        _ = terminate.recv() => { false },
        r = process.wait() => match r {
            Ok(None) => true,
            _ => false,
        },
    };

    let status = process.stop(true).await?;

    Ok((status, timedout))
}

impl Command {
    pub async fn spawn(&self) -> io::Result<Process> {
        // #[cfg(target_os = "linux")]
        // unsafe {
        //     libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0);
        // }

        let mut cmd = StdCommand::new(self.argv[0].as_str());

        cmd.args(&self.argv[1..]);

        cmd.stdout(if let Some(path) = self.stdout.as_ref() {
            fsutil::stdio_from(path, false).await?
        } else {
            Stdio::inherit()
        });

        cmd.stderr(if let Some(path) = self.stderr.as_ref() {
            fsutil::stdio_from(path, false).await?
        } else {
            Stdio::inherit()
        });

        cmd.env_clear().envs(env::vars().filter(|(key, _)| self.envs.contains(key)));

        // Put the child into a new process group.
        cmd.process_group(0);

        let child = process::Command::from(cmd).spawn()?;
        let id = child.id().expect("fetching the OS-assigned process id");
        Ok(Process { child, id, timeout: self.timeout })
    }
}

mod process_impl {
    use super::*;

    impl Process {
        /// If the process exits before the timeout has elapsed, then the completed
        /// result is returned. Otherwise, a `None` is returned.
        pub async fn wait(&mut self) -> io::Result<Option<ExitStatus>> {
            match self.timeout {
                // Always some because no timeout given.
                None => self.child.wait().await.map(Some),
                Some(dur) => match time::timeout(dur, self.child.wait()).await {
                    Err(_) => Ok(None),
                    Ok(status) => status.map(Some),
                },
            }
        }

        /// Stop the whole process group, and wait the child exits.
        #[tracing::instrument(skip(self))]
        pub async fn stop(&mut self, gracefully: bool) -> io::Result<ExitStatus> {
            if gracefully {
                self.killpg(libc::SIGTERM);
                time::sleep(Duration::from_millis(50)).await;
            }

            // TODO: Wait all descendant processes to ensure there are no children left behind.
            // Currently, the direct child is the only process to be waited before exiting.

            self.killpg(libc::SIGKILL);

            // Note that this loop is necessary even if the child process exits successfully
            // in order to 1) wait all descendant processes, 2) ensure there are no children left behind.
            loop {
                match self.child.try_wait()? {
                    // The exit status is not available at this time. The child may still be running.
                    // SIGKILL is sent just before entering the loop, but this happens because
                    // kill(2) just sends the signal to the given process(es).
                    // Once kill(2) returns, there is no guarantee that the signal has been delivered and handled.
                    None => continue,
                    Some(status) => break Ok(status),
                }
            }
        }

        fn killpg(&self, signal: libc::c_int) {
            let id = self.id as libc::c_int;
            unsafe {
                let killed = libc::killpg(id, signal);
                tracing::trace!(
                    signal,
                    killed,
                    errno = io::Error::last_os_error().raw_os_error().unwrap_or(0)
                );
            }
        }
    }

    // fn into_exit_status(status: StdExitStatus) -> ExitStatus {
    //     if status.success() {
    //         Ok(())
    //     } else if let Some(code) = status.code() {
    //         Err(ExitedUnsuccessfully(code))
    //     } else {
    //         // because `status.code()` returns `None`
    //         Err(KilledBySignal(status.signal().expect("WIFSIGNALED is true")))
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command<S: Into<String>>(program: Vec<S>) -> Command {
        Command {
            stdout: None,
            stderr: None,
            envs: vec![],
            timeout: None,
            argv: program.into_iter().map(|s| s.into()).collect(),
        }
    }

    fn sleep<S: Into<String>>(duration: S) -> Command {
        command(vec!["sleep".to_owned(), duration.into()])
    }

    #[tokio::test]
    async fn run_process() {
        assert!(command(vec!["date"]).run().await.is_ok());
        assert!(command(vec!["unknown_command"]).run().await.is_err());
        assert!(sleep("0.1").run().await.is_ok());
    }
}
