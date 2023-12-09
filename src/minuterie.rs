use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};

use crate::actor::Actor;
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


#[derive(Debug, Copy, Clone)]
pub struct Minuterie {
    last_instant: Instant,
    last_state: State,
    timeout: Duration,
}

impl Minuterie {
    pub(crate) fn new(timeout: Duration) -> Minuterie {
        Minuterie {
            last_instant: Instant::now() - timeout,
            last_state: INACTIVE,
            timeout,
        }
    }

    fn wait_for_timeout(&self) {
        let timeout_instant = self.last_instant + self.timeout;
        let sleep_duration = timeout_instant.saturating_duration_since(Instant::now());
        println!("sleep_duration {}", sleep_duration.as_millis());
        thread::sleep(sleep_duration);
    }

    fn get_current_state(&self) -> State {
        if Instant::now().duration_since(self.last_instant) < self.timeout {
            ACTIVE
        } else {
            INACTIVE
        }
    }

    fn publish(&mut self, tx: &Sender<Event>) -> anyhow::Result<()> {
        let current_state = self.get_current_state();
        // println!("Current state: {:?}", current_state);
        if current_state != self.last_state {
            tx.send(Event::from(MINUTERIE.to_string(), current_state))?;
        }
        self.last_state = current_state;
        Ok(())
    }
}

pub fn acquire_state(actor: &MinuterieActor) -> Minuterie {
    let data = actor.state.lock().unwrap();
    *data
}

type MinuterieActor = Actor<Minuterie, Event, Event>;

pub(crate) fn receiver(actor: MinuterieActor) -> anyhow::Result<()> {
    while let Ok(event) = actor.receive() {
        // println!("Received: {:?}", event);
        {
            let mut minuterie = actor.state.lock().unwrap();
            minuterie.last_instant = Instant::now();
            minuterie.publish(&actor.tx)?;
        }
        actor.notify_all();
    }
    Ok(())
}

pub(crate) fn timeout_clock(actor: MinuterieActor) -> anyhow::Result<()> {
    while !actor.is_stopped() {
        acquire_state(&actor).wait_for_timeout();
        let mut minuterie = acquire_state(&actor);
        minuterie.publish(&actor.tx)?;
        actor.park();
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::ops::Add;
    use std::thread;
    use std::time::{Duration, Instant};
    use crate::actor::Actor;
    use crate::minuterie::{Event, Minuterie, receiver, State, timeout_clock};

    #[test]
    fn minuterie_timing() {
        let launch = Instant::now();
        let input_events: Vec<Event> = [100u64, 500, 2000]
            .iter()
            .map(|t| Event {
                instant: launch.add(Duration::from_millis(*t)),
                topic: format!("input"),
                content: State::ACTIVE,
            }).collect();

        let actor = Actor::new(
            Minuterie::new(Duration::from_secs(1)),
            receiver,
            timeout_clock,
        );

        let tx = actor.tx;
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

        let rx = actor.rx;
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


//
//
// impl Minuterie {
//     pub fn start(timeout: Duration) -> (Sender<Event>, Receiver<Event>) {
//         let (input_tx, input_rx) = mpsc::channel();
//         let (output_tx, output_rx) = mpsc::channel();
//
//         // create a minuterie with elapsed timeout so it is not currently active
//         let mut minuterie = Minuterie {
//             last_instant: Arc::new(Mutex::new(Instant::now() - timeout)),
//             last_state: Arc::new(Mutex::new(State::ACTIVE)),
//             timeout,
//             rx: Some(input_rx),
//             tx: output_tx,
//             stopped: Arc::new(Mutex::new(false)),
//         };
//
//         thread::spawn(move || {
//             minuterie.run()
//         });
//         (input_tx, output_rx)
//     }
//
//     fn get_current_state(&self) -> State {
//         if let Ok(last_instant) = self.last_instant.lock() {
//             if Instant::now().duration_since(*last_instant) < self.timeout {
//                 ACTIVE
//             } else {
//                 INACTIVE
//             }
//         } else {
//             INACTIVE
//         }
//     }
//
//     fn get_last_state(&self) -> State {
//         let mut last_state = self.last_state.lock().unwrap();
//         *last_state
//     }
//
//     fn set_last_state(&self, state: State) {
//         let mut last_state = self.last_state.lock().unwrap();
//         *last_state = state;
//     }
//
//     fn get_last_instant(&self) -> Instant {
//         let mut last_instant = self.last_instant.lock().unwrap();
//         *last_instant
//     }
//
//     fn set_last_instant(&self, instant: Instant) {
//         let mut last_instant = self.last_instant.lock().unwrap();
//         *last_instant = instant;
//     }
//
//     fn stop(&mut self) {
//         let mut stopped = self.stopped.lock().unwrap();
//         *stopped = true;
//     }
//
//     fn is_stopped(&self) -> bool {
//         let stopped = self.stopped.lock().unwrap();
//         *stopped
//     }
//
//     fn publish(&mut self) -> anyhow::Result<()> {
//         let current_state = self.get_current_state();
//         if current_state != self.get_last_state() {
//             self.tx.send(Event::from(MINUTERIE.to_string(), current_state))?;
//         }
//         self.set_last_state(current_state);
//         Ok(())
//     }
//
//     fn receive(&self) -> anyhow::Result<Event> {
//         if let Some(rx) = &self.rx {
//             Ok(rx.recv()?)
//         } else {
//             Err(anyhow!("Could not receive event!"))
//         }
//     }
//
//     fn wait_for_timeout(&self) {
//         let timeout_instant = if let Ok(last_instant) = self.last_instant.lock() {
//             *last_instant + self.timeout
//         } else {
//             Instant::now() + Duration::from_millis(100)
//         };
//
//         thread::sleep(timeout_instant.saturating_duration_since(Instant::now()));
//     }
//
//     fn timeout_counter(&mut self) -> anyhow::Result<()> {
//         self.publish()?;
//
//         while !self.is_stopped() {
//             println!("Waiting for timeout...");
//             self.wait_for_timeout();
//             self.publish()?;
//         }
//
//         anyhow::Ok(())
//     }
//
//     fn event_receiver(&mut self) -> anyhow::Result<()> {
//         while let Ok(event) = self.receive() {
//             self.set_last_instant(event.instant);
//             self.publish()?;
//         }
//         anyhow::Ok(())
//     }
//
//     fn run(mut self) -> anyhow::Result<()> {
//         let timeout_counter = {
//             let mut minuterie = self.clone();
//             thread::spawn(move || {
//                 minuterie.timeout_counter()
//             })
//         };
//
//         let receiver = thread::spawn(move || {
//             let result = self.event_receiver();
//             self.stop();
//             result
//         });
//
//
//         let timeout_counter_result = timeout_counter.join();
//         let receiver_result = receiver.join();
//
//         match (timeout_counter_result, receiver_result) {
//             (Ok(_), Ok(_)) => Ok(()),
//             _ => Err(anyhow::anyhow!("Minuterie stopped"))
//         }
//     }
// }