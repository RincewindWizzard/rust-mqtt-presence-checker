use std::ops::Add;
use std::thread;
use std::time::{Duration, Instant};
use crate::actor::Actor;
use crate::minuterie::{Event, Minuterie, receiver, State, timeout_clock};

mod minuterie;
mod actor;


fn main() {
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

}

