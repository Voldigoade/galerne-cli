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
    _stdout_handle: Option<JoinHandle<()>>,
    _stderr_handle: Option<JoinHandle<()>>,
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
            _stdout_handle: stdout_handle,
            _stderr_handle: stderr_handle,
        })
    }

    /// Non-blocking query to check if the child process has exited.
    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        self.child.try_wait()
    }

    /// Blocks until the child process terminates.
    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.child.wait()
    }

    /// Forcibly kills the child process.
    pub fn kill(&mut self) -> io::Result<()> {
        self.child.kill()
    }

    /// Sends an interrupt signal (SIGINT on Unix / Ctrl+C on Windows) to the child.
    pub fn send_interrupt(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            let pid = self.child.id() as libc::pid_t;
            let ret = unsafe { libc::kill(pid, libc::SIGINT) };
            if ret != 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            // Windows fallback
            self.child.kill()
        }
    }
}
