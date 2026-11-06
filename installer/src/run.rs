//! Every external command and every file the installer touches goes through one `Runner`: `Real` on a live system,
//! `Fake` in the tests and the demo, so neither ever needs root or a disk.

use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// One external command: program, arguments and optional stdin. Stdin carries secrets (the password) and is never logged.
#[derive(Clone, Debug, Default)]
pub struct Cmd {
    pub program: String,
    pub args: Vec<String>,
    pub stdin: Option<String>,
}

impl Cmd {
    /// A command with no arguments yet.
    pub fn new(program: &str) -> Self {
        Self {
            program: program.into(),
            ..Self::default()
        }
    }

    /// Appends arguments.
    pub fn args<I: IntoIterator<Item = S>, S: AsRef<str>>(mut self, args: I) -> Self {
        self.args
            .extend(args.into_iter().map(|a| a.as_ref().to_string()));
        self
    }

    /// Feeds `text` to the command's stdin.
    pub fn stdin(mut self, text: String) -> Self {
        self.stdin = Some(text);
        self
    }

    /// The command line as the log shows it, arguments quoted when they hold spaces; stdin is left out.
    pub fn line(&self) -> String {
        let quote = |a: &String| {
            if a.is_empty() || a.contains([' ', '"', '\'']) {
                format!("{a:?}")
            } else {
                a.clone()
            }
        };
        std::iter::once(&self.program)
            .chain(&self.args)
            .map(quote)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The short form a failure names: the program and its last argument ("mkfs.btrfs /dev/nvme0n1p5").
    pub fn summary(&self) -> String {
        match self.args.last() {
            Some(last) => format!("{} {last}", self.program),
            None => self.program.clone(),
        }
    }
}

/// What a finished command left: its exit code (-1 when a signal ended it) and its output.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Output {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl Output {
    /// A successful run that printed `stdout`.
    pub fn ok(stdout: &str) -> Self {
        Self {
            code: 0,
            stdout: stdout.into(),
            stderr: String::new(),
        }
    }

    /// A failed run with exit `code` and `stderr`.
    pub fn fail(code: i32, stderr: &str) -> Self {
        Self {
            code,
            stdout: String::new(),
            stderr: stderr.into(),
        }
    }
}

/// Runs commands and touches files on behalf of the installer.
pub trait Runner: Send + Sync {
    /// Runs `cmd` to the end and returns its output.
    fn run(&self, cmd: &Cmd) -> io::Result<Output>;
    /// Runs `cmd`, handing each stdout line (split at `\r` or `\n`) to `line` as it arrives; a set `stop` ends it.
    fn stream(
        &self,
        cmd: &Cmd,
        line: &mut dyn FnMut(&str),
        stop: &AtomicBool,
    ) -> io::Result<Output>;
    /// Reads a text file.
    fn read(&self, path: &Path) -> io::Result<String>;
    /// Writes a text file (mode 0644), creating its parents; `owner` also owns the file and its parent directory.
    fn write(&self, path: &Path, text: &str, owner: Option<(u32, u32)>) -> io::Result<()>;
    /// Removes a file; a missing one is not an error.
    fn remove(&self, path: &Path) -> io::Result<()>;
}

/// The runner on a live system.
pub struct Real;

impl Runner for Real {
    fn run(&self, cmd: &Cmd) -> io::Result<Output> {
        let mut child = Command::new(&cmd.program)
            .args(&cmd.args)
            .stdin(if cmd.stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let (Some(text), Some(mut pipe)) = (&cmd.stdin, child.stdin.take()) {
            pipe.write_all(text.as_bytes())?;
        }
        let out = child.wait_with_output()?;
        Ok(Output {
            code: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).into(),
            stderr: String::from_utf8_lossy(&out.stderr).into(),
        })
    }

    fn stream(
        &self,
        cmd: &Cmd,
        line: &mut dyn FnMut(&str),
        stop: &AtomicBool,
    ) -> io::Result<Output> {
        let mut child = Command::new(&cmd.program)
            .args(&cmd.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let mut stdout = child.stdout.take().expect("piped stdout");
        let mut stderr = child.stderr.take().expect("piped stderr");
        let errors = thread::spawn(move || {
            let mut s = String::new();
            let _ = stderr.read_to_string(&mut s);
            s
        });
        // A blocking read cannot see `stop`, so a watcher ends the child, which closes stdout.
        let child = Arc::new(Mutex::new(child));
        let finished = Arc::new(AtomicBool::new(false));
        thread::scope(|scope| {
            let (watched, done) = (child.clone(), finished.clone());
            scope.spawn(move || {
                while !done.load(Ordering::Relaxed) {
                    if stop.load(Ordering::Relaxed) {
                        let _ = watched.lock().expect("child lock").kill();
                        return;
                    }
                    thread::sleep(Duration::from_millis(50));
                }
            });
            let mut buf = [0u8; 4096];
            let mut pending = Vec::new();
            let mut last = String::new();
            loop {
                let n = match stdout.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                for &b in &buf[..n] {
                    if b == b'\r' || b == b'\n' {
                        if !pending.is_empty() {
                            last = String::from_utf8_lossy(&pending).into();
                            line(&last);
                            pending.clear();
                        }
                    } else {
                        pending.push(b);
                    }
                }
            }
            if !pending.is_empty() {
                last = String::from_utf8_lossy(&pending).into();
                line(&last);
            }
            finished.store(true, Ordering::Relaxed);
            let status = child.lock().expect("child lock").wait()?;
            Ok(Output {
                code: status.code().unwrap_or(-1),
                stdout: last,
                stderr: errors.join().unwrap_or_default(),
            })
        })
    }

    fn read(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn write(&self, path: &Path, text: &str, owner: Option<(u32, u32)>) -> io::Result<()> {
        let parent = path.parent().unwrap_or(Path::new("/"));
        fs::create_dir_all(parent)?;
        fs::write(path, text)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o644))?;
        if let Some((uid, gid)) = owner {
            std::os::unix::fs::chown(parent, Some(uid), Some(gid))?;
            std::os::unix::fs::chown(path, Some(uid), Some(gid))?;
        }
        Ok(())
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        match fs::remove_file(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            r => r,
        }
    }
}

/// A fake command's effect on the fake's files.
pub type Effect = fn(&Cmd, &mut HashMap<PathBuf, String>);

/// What the fake has seen and holds; shared, so a test can inspect it while the install thread owns a clone.
#[derive(Default)]
pub struct FakeState {
    /// Rules: a command line that starts with the prefix gets the output. The first match wins.
    pub rules: Vec<(String, Output)>,
    /// Lines a streamed command prints, in order, `delay` apart.
    pub stream_lines: Vec<String>,
    pub delay: Duration,
    /// Files `read` can see, and what `write` and `remove` did to them.
    pub files: HashMap<PathBuf, String>,
    pub owners: HashMap<PathBuf, (u32, u32)>,
    /// Effects: a command whose line starts with the prefix also changes the files, as the real one would.
    pub effects: Vec<(String, Effect)>,
    /// Every command line run, and those no rule matched.
    pub calls: Vec<String>,
    pub stdins: Vec<String>,
    pub unexpected: Vec<String>,
}

/// The runner for the tests and the demo: answers from rules and an in-memory file table, touches nothing real.
#[derive(Clone, Default)]
pub struct Fake(pub Arc<Mutex<FakeState>>);

impl Fake {
    /// Adds a rule: commands whose line starts with `prefix` return `out`.
    pub fn on(&self, prefix: &str, out: Output) -> &Self {
        self.state().rules.push((prefix.into(), out));
        self
    }

    /// Adds a file `read` can see.
    pub fn file(&self, path: &str, text: &str) -> &Self {
        self.state().files.insert(path.into(), text.into());
        self
    }

    /// Locks the shared state.
    pub fn state(&self) -> std::sync::MutexGuard<'_, FakeState> {
        self.0.lock().expect("fake runner lock")
    }

    /// Commands run so far whose line starts with `prefix`.
    #[cfg(test)]
    pub fn calls(&self, prefix: &str) -> Vec<String> {
        self.state()
            .calls
            .iter()
            .filter(|c| c.starts_with(prefix))
            .cloned()
            .collect()
    }

    fn answer(&self, cmd: &Cmd) -> Output {
        let mut s = self.state();
        let line = cmd.line();
        s.calls.push(line.clone());
        if let Some(text) = &cmd.stdin {
            s.stdins.push(text.clone());
        }
        let effects: Vec<_> = s
            .effects
            .iter()
            .filter(|(p, _)| line.starts_with(p.as_str()))
            .map(|(_, e)| *e)
            .collect();
        for e in effects {
            e(cmd, &mut s.files);
        }
        match s.rules.iter().find(|(p, _)| line.starts_with(p.as_str())) {
            Some((_, out)) => out.clone(),
            None => {
                s.unexpected.push(line);
                Output::fail(127, "not expected by the fake runner")
            }
        }
    }
}

impl Runner for Fake {
    fn run(&self, cmd: &Cmd) -> io::Result<Output> {
        Ok(self.answer(cmd))
    }

    fn stream(
        &self,
        cmd: &Cmd,
        line: &mut dyn FnMut(&str),
        stop: &AtomicBool,
    ) -> io::Result<Output> {
        let out = self.answer(cmd);
        let (lines, delay) = {
            let s = self.state();
            (s.stream_lines.clone(), s.delay)
        };
        for l in &lines {
            if stop.load(Ordering::Relaxed) {
                return Ok(Output::fail(-1, "stopped"));
            }
            thread::sleep(delay);
            line(l);
        }
        Ok(out)
    }

    fn read(&self, path: &Path) -> io::Result<String> {
        self.state()
            .files
            .get(path)
            .cloned()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, path.display().to_string()))
    }

    fn write(&self, path: &Path, text: &str, owner: Option<(u32, u32)>) -> io::Result<()> {
        let mut s = self.state();
        s.files.insert(path.into(), text.into());
        if let Some(o) = owner {
            s.owners.insert(path.into(), o);
        }
        Ok(())
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        self.state().files.remove(path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_quotes_spaces_and_summary_takes_the_last_argument() {
        let c = Cmd::new("mkfs.btrfs").args(["-f", "-L", "stackific", "/dev/nvme0n1p5"]);
        assert_eq!(c.summary(), "mkfs.btrfs /dev/nvme0n1p5");
        assert_eq!(
            Cmd::new("useradd")
                .args(["--comment", "Ada Lovelace"])
                .line(),
            r#"useradd --comment "Ada Lovelace""#
        );
    }

    #[test]
    fn real_stream_splits_carriage_returns_and_stops() {
        let mut seen = Vec::new();
        let out = Real
            .stream(
                &Cmd::new("printf").args(["a\rb\nc"]),
                &mut |l| seen.push(l.to_string()),
                &AtomicBool::new(false),
            )
            .unwrap();
        assert_eq!(
            (seen, out.code),
            (vec!["a".to_string(), "b".into(), "c".into()], 0)
        );
        let out = Real
            .stream(
                &Cmd::new("sleep").args(["5"]),
                &mut |_| {},
                &AtomicBool::new(true),
            )
            .unwrap();
        assert_ne!(out.code, 0);
    }

    #[test]
    fn fake_records_unexpected_commands() {
        let f = Fake::default();
        f.on("udevadm settle", Output::ok(""));
        assert_eq!(
            f.run(&Cmd::new("udevadm").args(["settle"])).unwrap().code,
            0
        );
        assert_eq!(f.run(&Cmd::new("rm").args(["-rf", "/"])).unwrap().code, 127);
        assert_eq!(f.state().unexpected, vec!["rm -rf /".to_string()]);
    }
}
