use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};

use std::{fmt, thread};
use std::time::{Duration, Instant};


use crate::minuterie::State::{ACTIVE, INACTIVE};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateChange {
    /// Timestamp of the state change
    pub instant: Instant,
    /// New state
    pub state: State,
}


/// Represents a measurement of presence.
/// This could be a ping response from your mobile, a movement or door sensor
/// or everything else which senses your presence.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Heartbeat {
    pub instant: Instant,
}

impl Default for Heartbeat {
    fn default() -> Self {
        Heartbeat {
            instant: Instant::now(),
        }
    }
}


impl From<State> for StateChange {
    fn from(state: State) -> Self {
        StateChange {
            instant: Instant::now(),
            state,
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    ACTIVE,
    INACTIVE,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            State::ACTIVE => write!(f, "ACTIVE"),
            State::INACTIVE => write!(f, "INACTIVE"),
        }
    }
}

#[derive(Debug)]
pub struct Minuterie {
    last_instant: Instant,
    last_state: State,
    timeout: Duration,
    rx: Receiver<Heartbeat>,
    tx: Sender<StateChange>,
}

impl Minuterie {
    pub fn new(timeout: Duration) -> (Sender<Heartbeat>, Receiver<StateChange>) {
        let (input_tx, input_rx) = mpsc::channel();
        let (output_tx, output_rx) = mpsc::channel();

        let mut minuterie = Minuterie {
            last_instant: Instant::now() - timeout,
            last_state: INACTIVE,
            timeout,
            rx: input_rx,
            tx: output_tx,
        };

        thread::spawn(move || {
            minuterie.run()
        });

        (input_tx, output_rx)
    }

    fn recv(&self) -> Result<Heartbeat, RecvTimeoutError> {
        let timer = self.get_duration_until_timeout();
        if timer > Duration::from_secs(0) {
            self.rx.recv_timeout(timer)
        } else {
            Ok(self.rx.recv()?)
        }
    }

    fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let heartbeat = self.recv();
            if let Ok(heartbeat) = heartbeat {
                self.last_instant = heartbeat.instant;
            } else if let Err(err) = heartbeat {
                if err == RecvTimeoutError::Disconnected {
                    // when the channel is disconnected, stop the loop
                    break;
                }
            }

            self.publish_state()?;
        }

        Ok(())
    }

    fn get_duration_until_timeout(&self) -> Duration {
        let timeout_instant = self.last_instant + self.timeout;
        timeout_instant.saturating_duration_since(Instant::now())
    }

    fn get_current_state(&self) -> State {
        if Instant::now().duration_since(self.last_instant) < self.timeout {
            ACTIVE
        } else {
            INACTIVE
        }
    }

    /// Publishes changes to the state to the output channel
    fn publish_state(&mut self) -> anyhow::Result<()> {
        let current_state = self.get_current_state();
        if current_state != self.last_state {
            self.tx.send(StateChange::from(current_state))?;
        }
        self.last_state = current_state;
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::ops::Add;
    use std::thread;
    use std::time::{Duration, Instant};
    use crate::minuterie::{Minuterie, State, Heartbeat};
    use crate::minuterie::State::{ACTIVE, INACTIVE};

    fn assert_timing(timeout: u64, input_events: &[u64], expected: &[(u64, State)]) {
        let launch = Instant::now();
        let input_events: Vec<Heartbeat> = input_events
            .iter()
            .map(|t| Heartbeat {
                instant: launch.add(Duration::from_millis(*t)),
            }).collect();

        let (tx, rx) = Minuterie::new(Duration::from_millis(timeout));

        thread::spawn(move || {
            for event in input_events {
                let now = Instant::now();
                let wait_duration = event.instant.saturating_duration_since(now);
                thread::sleep(wait_duration);
                assert!(event.instant < Instant::now());
                tx.send(event).unwrap();
            }

            // wait for timeout to expire
            thread::sleep(Duration::from_millis(timeout * 2));
        });

        let mut events = vec![];
        while let Ok(event) = rx.recv() {
            events.push(event.clone());
            println!(
                "Event({}, {:?})",
                event.instant.duration_since(launch).as_millis(),
                event.state
            )
        }

        assert_eq!(events.len(), expected.len());
        for i in 0..expected.len() {
            let actual = &events[i];
            let (expected_instant, expected_state) = &expected[i];

            assert_eq!(*expected_instant as u128, actual.instant.duration_since(launch).as_millis());
            assert_eq!(expected_state, &actual.state);
        }
    }

    #[test]
    fn minuterie_timing() {
        assert_timing(
            100,
            &[10, 50, 200, 500, 610, 611, 612, 613],
            &[
                (10, ACTIVE),
                (150, INACTIVE),
                (200, ACTIVE),
                (300, INACTIVE),
                (500, ACTIVE),
                (600, INACTIVE),
                (610, ACTIVE),
                (713, INACTIVE),
            ],
        );
    }
}

