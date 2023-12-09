use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use std::time::{Duration, Instant};
use anyhow::{anyhow};
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
    last_instant: Arc<Mutex<Instant>>,
    last_state: Arc<Mutex<State>>,
    timeout: Duration,
    rx: Option<Receiver<Event>>,
    tx: Sender<Event>,
    stopped: Arc<Mutex<bool>>,
}

impl Clone for Minuterie {
    fn clone(&self) -> Self {
        Minuterie {
            last_instant: self.last_instant.clone(),
            last_state: self.last_state.clone(),
            timeout: self.timeout,
            rx: None,
            tx: self.tx.clone(),
            stopped: self.stopped.clone(),
        }
    }
}


impl Minuterie {
    pub fn start(timeout: Duration) -> (Sender<Event>, Receiver<Event>) {
        let (input_tx, input_rx) = mpsc::channel();
        let (output_tx, output_rx) = mpsc::channel();

        // create a minuterie with elapsed timeout so it is not currently active
        let mut minuterie = Minuterie {
            last_instant: Arc::new(Mutex::new(Instant::now() - timeout)),
            last_state: Arc::new(Mutex::new(State::ACTIVE)),
            timeout,
            rx: Some(input_rx),
            tx: output_tx,
            stopped: Arc::new(Mutex::new(false)),
        };

        thread::spawn(move || {
            minuterie.run()
        });
        (input_tx, output_rx)
    }

    fn get_current_state(&self) -> State {
        if let Ok(last_instant) = self.last_instant.lock() {
            if Instant::now().duration_since(*last_instant) < self.timeout {
                ACTIVE
            } else {
                INACTIVE
            }
        } else {
            INACTIVE
        }
    }

    fn get_last_state(&self) -> State {
        let mut last_state = self.last_state.lock().unwrap();
        *last_state
    }

    fn set_last_state(&self, state: State) {
        let mut last_state = self.last_state.lock().unwrap();
        *last_state = state;
    }

    fn get_last_instant(&self) -> Instant {
        let mut last_instant = self.last_instant.lock().unwrap();
        *last_instant
    }

    fn set_last_instant(&self, instant: Instant) {
        let mut last_instant = self.last_instant.lock().unwrap();
        *last_instant = instant;
    }

    fn stop(&mut self) {
        let mut stopped = self.stopped.lock().unwrap();
        *stopped = true;
    }

    fn is_stopped(&self) -> bool {
        let stopped = self.stopped.lock().unwrap();
        *stopped
    }

    fn publish(&mut self) -> anyhow::Result<()> {
        let current_state = self.get_current_state();
        if current_state != self.get_last_state() {
            self.tx.send(Event::from(MINUTERIE.to_string(), current_state))?;
        }
        self.set_last_state(current_state);
        Ok(())
    }

    fn receive(&self) -> anyhow::Result<Event> {
        if let Some(rx) = &self.rx {
            Ok(rx.recv()?)
        } else {
            Err(anyhow!("Could not receive event!"))
        }
    }

    fn wait_for_timeout(&self) {
        let timeout_instant = if let Ok(last_instant) = self.last_instant.lock() {
            *last_instant + self.timeout
        } else {
            Instant::now() + Duration::from_millis(100)
        };

        thread::sleep(timeout_instant.saturating_duration_since(Instant::now()));
    }

    fn timeout_counter(&mut self) -> anyhow::Result<()> {
        self.publish()?;

        while !self.is_stopped() {
            println!("Waiting for timeout...");
            self.wait_for_timeout();
            self.publish()?;
        }

        anyhow::Ok(())
    }

    fn event_receiver(&mut self) -> anyhow::Result<()> {
        while let Ok(event) = self.receive() {
            self.set_last_instant(event.instant);
            self.publish()?;
        }
        anyhow::Ok(())
    }

    fn run(mut self) -> anyhow::Result<()> {
        let timeout_counter = {
            let mut minuterie = self.clone();
            thread::spawn(move || {
                minuterie.timeout_counter()
            })
        };

        let receiver = thread::spawn(move || {
            let result = self.event_receiver();
            self.stop();
            result
        });


        let timeout_counter_result = timeout_counter.join();
        let receiver_result = receiver.join();

        match (timeout_counter_result, receiver_result) {
            (Ok(_), Ok(_)) => Ok(()),
            _ => Err(anyhow::anyhow!("Minuterie stopped"))
        }
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