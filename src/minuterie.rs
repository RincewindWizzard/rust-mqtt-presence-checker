use std::fmt::{Display, Formatter};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SendError};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use crate::minuterie::State::{ACTIVE, INACTIVE};

const MINUTERIE: &str = "Minuterie";

pub struct Event {
    pub instant: Instant,
    pub topic: String,
    pub content: State,
}

impl Event {
    pub fn from(topic: String, content: State) -> Event {
        Event {
            instant: Instant::now(),
            topic,
            content,
        }
    }
}

#[derive(Debug)]
enum State {
    ACTIVE,
    INACTIVE,
}

pub struct Minuterie {
    last_event: Instant,
    timeout: Duration,
    rx: Receiver<Event>,
    tx: Sender<Event>,
}


trait Publisher {
    fn publish(&self, state: State) -> Result<(), SendError<Event>>;
}

impl Publisher for Sender<Event> {
    fn publish(&self, state: State) -> Result<(), SendError<Event>> {
        self.send(Event::from(MINUTERIE.to_string(), state))
    }
}


impl Minuterie {
    pub fn start(timeout: Duration) -> (Sender<Event>, Receiver<Event>) {
        let (input_tx, input_rx) = mpsc::channel();
        let (output_tx, output_rx) = mpsc::channel();

        // create a minuterie with elapsed timeout so it is not currently active
        let mut minuterie = Minuterie {
            last_event: Instant::now() - timeout,
            timeout,
            rx: input_rx,
            tx: output_tx,
        };

        thread::spawn(move || {
            minuterie.run()
        });
        (input_tx, output_rx)
    }

    fn current_state(&self) -> State {
        if Instant::now().duration_since(self.last_event) < self.timeout {
            ACTIVE
        } else {
            INACTIVE
        }
    }

    fn run(&mut self) -> Result<(), SendError<Event>> {
        let timeout = &self.timeout;

        self.tx.publish(self.current_state())?;

        while let Ok(event) = self.rx.recv() {
            self.last_event = event.instant;
            self.tx.publish(self.current_state())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::{Duration, Instant};
    use rand::Rng;
    use crate::minuterie::{Event, Minuterie};
    use crate::minuterie::State::ACTIVE;

    #[test]
    fn minuterie_timing() {
        let launch = Instant::now();
        let (tx, rx) = Minuterie::start(Duration::from_secs(1));

        thread::spawn(move || {
            for i in 1..10 {
                let mut rng = rand::thread_rng();
                thread::sleep(Duration::from_millis(rng.gen_range(0..100)));
                tx.send(Event::from(format!("Test-{i}"), ACTIVE)).unwrap();
            }
        });

        while let Ok(event) = rx.recv() {
            println!(
                "Event({}, {}, {:?})",
                event.instant.duration_since(launch).as_millis(),
                event.topic,
                event.content
            )
        }
    }
}