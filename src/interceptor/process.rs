//! Child process spawning, bounded log buffering, and exit code propagation.

use std::collections::VecDeque;
use std::io::{self, BufRead, BufReader};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

pub const DEFAULT_MAX_LOG_LINES: usize = 2000;

/// Bounded, thread-safe FIFO log buffer preventing unbounded memory growth.
#[derive(Clone, Debug)]
pub struct OutputBuffer {
    capacity: usize,
    lines: Arc<Mutex<VecDeque<String>>>,
}

impl OutputBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            lines: Arc::new(Mutex::new(VecDeque::with_capacity(capacity.min(512)))),
        }
    }

    pub fn push_line(&self, line: String) {
        if let Ok(mut buf) = self.lines.lock() {
            if buf.len() >= self.capacity {
                buf.pop_front();
            }
            buf.push_back(line);
        }
    }

    pub fn drain_lines(&self) -> Vec<String> {
        if let Ok(mut buf) = self.lines.lock() {
            buf.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    pub fn clone_lines(&self) -> Vec<String> {
        if let Ok(buf) = self.lines.lock() {
            buf.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

impl Default for OutputBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_LOG_LINES)
    }
}

/// Supervises a child development process while buffering output.
pub struct ProcessSupervisor {
    child: Child,
    pub output: OutputBuffer,
    stdout_handle: Option<JoinHandle<()>>,
    stderr_handle: Option<JoinHandle<()>>,
}

impl ProcessSupervisor {
    /// Spawns the specified command and captures stdout/stderr into bounded buffers.
    pub fn spawn(args: &[String]) -> io::Result<Self> {
        if args.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot spawn empty command",
            ));
        }

        let mut cmd = Command::new(&args[0]);
        if args.len() > 1 {
            cmd.args(&args[1..]);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }

        let mut child = cmd.spawn()?;
        let output = OutputBuffer::default();

        let stdout_handle = if let Some(stdout_pipe) = child.stdout.take() {
            let out_clone = output.clone();
            Some(thread::spawn(move || {
                let reader = BufReader::new(stdout_pipe);
                for line in reader.lines().map_while(Result::ok) {
                    out_clone.push_line(line);
                }
            }))
        } else {
            None
        };

        let stderr_handle = if let Some(stderr_pipe) = child.stderr.take() {
            let out_clone = output.clone();
            Some(thread::spawn(move || {
                let reader = BufReader::new(stderr_pipe);
                for line in reader.lines().map_while(Result::ok) {
                    out_clone.push_line(line);
                }
            }))
        } else {
            None
        };

        Ok(Self {
            child,
            output,
            stdout_handle,
            stderr_handle,
        })
    }

    fn join_readers(&mut self) {
        if let Some(h) = self.stdout_handle.take() {
            let _ = h.join();
        }
        if let Some(h) = self.stderr_handle.take() {
            let _ = h.join();
        }
    }

    /// Non-blocking query to check if the child process has exited.
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        let status = self.child.try_wait()?;
        if status.is_some() {
            self.join_readers();
        }
        Ok(status)
    }

    /// Blocks until the child process terminates and flushes buffered output.
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        let status = self.child.wait()?;
        self.join_readers();
        Ok(status)
    }

    /// Forcibly kills the child process.
    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    /// Terminates or interrupts the supervised process (SIGINT to the process group on Unix; process-tree termination via taskkill on Windows).
    pub fn send_interrupt(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            let pid = self.child.id() as libc::pid_t;
            // Send SIGINT to the entire child process group (-pid)
            let ret = unsafe { libc::kill(-pid, libc::SIGINT) };
            if ret != 0 {
                // Fallback to signaling the single process
                let ret2 = unsafe { libc::kill(pid, libc::SIGINT) };
                if ret2 != 0 {
                    return Err(io::Error::last_os_error());
                }
            }
            Ok(())
        }
        #[cfg(windows)]
        {
            let pid = self.child.id();
            // On Windows, taskkill /PID <pid> /T /F terminates the entire process tree
            let status = Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .status();
            if status.is_err() || !status.unwrap().success() {
                // Fallback to basic process kill if taskkill fails
                let _ = self.child.kill();
            }
            Ok(())
        }
        #[cfg(all(not(unix), not(windows)))]
        {
            self.child.kill()
        }
    }
}
