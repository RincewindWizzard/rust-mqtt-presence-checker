use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use log::{debug, trace};

use crate::minuterie::Heartbeat;

/// Pings a host every interval.
/// Every response triggers a heartbeat which is send to the receiver
pub fn ping(host: String, interval: Duration) -> Receiver<Heartbeat> {
    let (tx, rx) = mpsc::channel();

    let _: JoinHandle<anyhow::Result<ExitStatus>> = thread::spawn(move || {
        let mut child = Command::new("ping")
            .arg("-i")// Wait interval seconds between sending each packet.
            .arg(interval.as_secs_f64().to_string())
            .arg("-D")
            .arg(&host)
            .stdout(Stdio::piped())
            .env("LANG", "en_US.UTF-8")
            .spawn()?;

        // Retrieve stdout as a handle to read line by line
        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);

            // Read lines from stdout while the command is running
            for line in reader.lines() {
                if let Ok(line) = line {
                    trace!("{}", line);
                    if line.starts_with("[") && !line.contains("Destination Host Unreachable") {
                        debug!("Heartbeat from {host}");
                        tx.send(Heartbeat::default())?;
                    }
                }
            }
        }
        Ok(child.wait()?)
    });

    rx
}



