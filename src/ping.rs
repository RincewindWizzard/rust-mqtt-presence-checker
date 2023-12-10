use std::{io, thread};
use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread::JoinHandle;
use std::time::{Duration, Instant, UNIX_EPOCH};
use regex::Regex;
use crate::minuterie::Heartbeat;



pub fn ping(host: String, interval: Duration) -> Receiver<Heartbeat> {

    let (tx, rx) = mpsc::channel();

    let result : JoinHandle<anyhow::Result<ExitStatus>> = thread::spawn(move || {
        let mut child = Command::new("ping")
            .arg("-i")// Wait interval seconds between sending each packet.
            .arg(interval.as_secs_f64().to_string())
            .arg("-D")
            .arg(host)
            .stdout(Stdio::piped())
            .spawn()?;

        // Retrieve stdout as a handle to read line by line
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);

            // Read lines from stdout while the command is running
            for line in reader.lines() {
                if let Ok(line) = line {
                    println!("{}", line);
                    if line.starts_with("[") {
                        tx.send(Heartbeat::default())?;
                    }
                }
            }
        }
        Ok(child.wait()?)
    });

    rx
}



