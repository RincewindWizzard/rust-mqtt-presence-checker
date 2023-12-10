
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};
use clap::Parser;
use directories::ProjectDirs;
use log::{debug, error, info, trace, warn};
use crate::args::ApplicationContext;
use crate::channel_utility::merge_channels;
use crate::minuterie::{Heartbeat, Minuterie};
use crate::ping::ping;

mod minuterie;
mod ping;
mod channel_utility;
mod args;





fn main() {
    let context = ApplicationContext::construct().unwrap();

    stderrlog::new()
        .module(module_path!())
        .quiet(context.args.quiet)
        .verbosity(context.args.verbose as usize + 1) // show warnings and above
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init().expect("Could not setup logging!");


    debug!("{:?}", context);

    // let ping_interval = Duration::from_millis(1000);
    // let ping_hosts = [
    //     "192.168.178.1",
    //     "192.168.178.2"
    // ];
    //
    // let ping_heartbeats: Vec<Receiver<Heartbeat>> = ping_hosts
    //     .iter()
    //     .map(|host| ping(host.to_string(), ping_interval))
    //     .collect();
    //
    //
    // let (minuterie_tx, state_changes) = Minuterie::new(Duration::from_millis(100));
    //
    // merge_channels(ping_heartbeats, minuterie_tx);
    //
    // while let Ok(state_change) = state_changes.recv() {
    //     println!(
    //         "{:>7}: {:?}",
    //         state_change.instant.duration_since(launch).as_millis(),
    //         state_change.state
    //     );
    // }
}

