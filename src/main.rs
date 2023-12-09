use std::fmt::Display;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use rand::Rng;
use crate::minuterie::{Event, Minuterie};

mod minuterie;


fn main() {
    let launch = Instant::now();
    let mut minuterie = Minuterie::new(Duration::from_secs(1));
    let (tx, rx) = minuterie.start();

}

