use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

struct ChildGuard(Child);

impl ChildGuard {
    fn kill_and_reap(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill_and_reap();
    }
}

fn stdout_lines(stdout: impl Read + Send + 'static) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut reader = BufReader::new(stdout);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) => {
                    if tx
                        .send(line.trim_end_matches(['\r', '\n']).to_string())
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    });
    rx
}

fn expect_line(child: &mut ChildGuard, rx: &Receiver<String>, expected: &str) {
    let line = match rx.recv_timeout(Duration::from_secs(8)) {
        Ok(line) => line,
        Err(e) => {
            child.kill_and_reap();
            panic!("waiting for {expected:?}: {e}");
        }
    };
    if line != expected {
        child.kill_and_reap();
        panic!("expected {expected:?}, received {line:?}");
    }
}

fn wait_for_exit(child: &mut ChildGuard) -> ExitStatus {
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        match child.0.try_wait() {
            Ok(Some(status)) => return status,
            Ok(None) => {}
            Err(e) => {
                child.kill_and_reap();
                panic!("check exiftool-rs exit status: {e}");
            }
        }
        if Instant::now() >= deadline {
            child.kill_and_reap();
            panic!("exiftool-rs did not exit after -stay_open False");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn stay_open_stdin_processes_incremental_commands() {
    let mut child = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_exiftool-rs"))
            .args(["-stay_open", "True", "-@", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start exiftool-rs"),
    );
    let stdout = stdout_lines(child.0.stdout.take().unwrap());
    let mut stderr = child.0.stderr.take().unwrap();
    let stderr_reader = thread::spawn(move || {
        let mut sink = std::io::sink();
        std::io::copy(&mut stderr, &mut sink).unwrap();
    });
    let mut stdin = child.0.stdin.take().unwrap();

    stdin.write_all(b"-ver\n-execute1\n").unwrap();
    stdin.flush().unwrap();
    expect_line(&mut child, &stdout, "13.60");
    expect_line(&mut child, &stdout, "{ready1}");

    stdin.write_all(b"-ver\n-execute2\n").unwrap();
    stdin.flush().unwrap();
    expect_line(&mut child, &stdout, "13.60");
    expect_line(&mut child, &stdout, "{ready2}");

    stdin.write_all(b"-stay_open\nFalse\n").unwrap();
    stdin.flush().unwrap();
    let status = wait_for_exit(&mut child);
    drop(stdin);
    assert!(status.success(), "exiftool-rs exited with {status}");
    stderr_reader.join().unwrap();
}

#[test]
fn stay_open_stdin_eof_exits_cleanly() {
    let mut child = ChildGuard(
        Command::new(env!("CARGO_BIN_EXE_exiftool-rs"))
            .args(["-stay_open", "True", "-@", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start exiftool-rs"),
    );
    drop(child.0.stdin.take().unwrap());
    let status = wait_for_exit(&mut child);
    assert!(status.success(), "exiftool-rs exited with {status}");
}
