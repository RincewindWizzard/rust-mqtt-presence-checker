use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub struct Event {
    pub instant: Instant,
    pub topic: String,
    pub content: String,
}

impl Event {
    pub fn from(topic: String, content: String) -> Event {
        Event {
            instant: Instant::now(),
            topic,
            content,
        }
    }
}

pub struct Minuterie {
    last_event: Instant,
    timeout: Duration,
}

impl Minuterie {
    pub fn new(timeout: Duration) -> Minuterie {
        // create a minuterie with elapsed timeout so it is not currently active
        Minuterie {
            last_event: Instant::now() - timeout,
            timeout
        }
    }

    fn run(&self, rx: Receiver<Event>, tx: Sender<Event>) {
        let timeout = &self.timeout;
        while let Ok(event) = rx.recv() {
            if let Err(error) = tx.send(event) {
                println!("Error {}", error);
                break;
            }
        }
    }

    pub fn start(self) -> (Sender<Event>, Receiver<Event>) {
        let (input_tx, input_rx) = mpsc::channel();
        let (output_tx, output_rx) = mpsc::channel();
        thread::spawn(move || {
            self.run(input_rx, output_tx)
        });
        (input_tx, output_rx)
    }

}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};
    use rand::Rng;
    use crate::minuterie::{Event, Minuterie};

    #[test]
    fn minuterie_timing() {
        let launch = Instant::now();
        let mut minuterie = Minuterie::new(Duration::from_secs(1));
        let (tx, rx) = minuterie.start();

        thread::spawn(move || {
            for i in 1..10 {
                let mut rng = rand::thread_rng();
                thread::sleep(Duration::from_millis(rng.gen_range(0..1000)));
                tx.send(Event::from(format!("Test-{i}"), format!("alive"))).unwrap();
            }
        });

        while let Ok(event) = rx.recv() {
            println!(
                "Event({}, {}, {})",
                event.instant.duration_since(launch).as_millis(),
                event.topic,
                event.content
            )
        }
    }
}