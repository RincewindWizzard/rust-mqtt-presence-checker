use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SendError};
use std::thread;

use std::time::{Duration, Instant};
use crate::minuterie::State::{ACTIVE, INACTIVE};

const MINUTERIE: &str = "Minuterie";

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    ACTIVE,
    INACTIVE,
}

pub struct Minuterie {
    last_event: Instant,
    last_state: State,
    timeout: Duration,
    rx: Receiver<Event>,
    tx: Sender<Event>,
}


impl Minuterie {
    pub fn start(timeout: Duration) -> (Sender<Event>, Receiver<Event>) {
        let (input_tx, input_rx) = mpsc::channel();
        let (output_tx, output_rx) = mpsc::channel();

        // create a minuterie with elapsed timeout so it is not currently active
        let mut minuterie = Minuterie {
            last_event: Instant::now() - timeout,
            last_state: State::ACTIVE,
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

    fn publish(&mut self) -> Result<(), SendError<Event>> {
        let current_state = self.current_state();
        if current_state != self.last_state {
            self.tx.send(Event::from(MINUTERIE.to_string(), current_state))?;
        }
        self.last_state = current_state;
        Ok(())
    }

    fn run(&mut self) -> Result<(), SendError<Event>> {
        self.publish()?;

        while let Ok(event) = self.rx.recv() {
            self.last_event = event.instant;
            self.publish()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;
    use std::thread;
    use std::time::{Duration, Instant};
    use crate::minuterie::{Event, Minuterie, State};

    #[test]
    fn minuterie_timing() {
        let launch = Instant::now();
        let (tx, rx) = Minuterie::start(Duration::from_secs(1));


        let input_events: Vec<Event> = [100u64, 500, 2000]
            .iter()
            .map(|t| Event {
                instant: launch.add(Duration::from_millis(*t)),
                topic: format!("input"),
                content: State::ACTIVE,
            }).collect();


        thread::spawn(move || {
            for event in input_events {
                let now = Instant::now();
                let wait_duration = event.instant.saturating_duration_since(now);
                thread::sleep(wait_duration);
                assert!(event.instant < Instant::now());
                tx.send(event).unwrap();
            }

            // wait for timeout to expire
            thread::sleep(Duration::from_secs(2));
        });

        let mut events = vec![];
        while let Ok(event) = rx.recv() {
            events.push(event.clone());
            println!(
                "Event({}, {}, {:?})",
                event.instant.duration_since(launch).as_millis(),
                event.topic,
                event.content
            )
        }

        assert_eq!(events.len(), 3);
    }
}