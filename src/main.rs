use std::sync::mpsc;
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
use rumqttc::{MqttOptions, Client, QoS};
use crate::mqtt::{mqtt_connect};

mod minuterie;
mod ping;
mod channel_utility;
mod args;
mod mqtt;


fn main() {
    let context = ApplicationContext::construct().unwrap();

    stderrlog::new()
        .module(module_path!())
        .quiet(context.args.quiet)
        .verbosity(context.args.verbose as usize + 1) // show warnings and above
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init().expect("Could not setup logging!");


    debug!("{:?}", context);

    let mut heartbeats = vec![];
    for probe in context.config.ping.hosts {
        let ping_heartbeat = ping(probe.host.clone(), probe.get_interval());
        heartbeats.push(ping_heartbeat);
    }

    let (tx_state, mqtt_heartbeat) = mqtt_connect(
        MqttOptions::from(&context.config.mqtt),
        &context.config.mqtt.heartbeat_topic,
        &context.config.mqtt.publish_topic,
    );

    heartbeats.push(mqtt_heartbeat);

    let (minuterie_tx, state_changes) = Minuterie::new(context.config.minuterie.get_timeout());

    // connecting the actors
    merge_channels(heartbeats, minuterie_tx);

    // if this breaks, shutdown the daemon
    while let Ok(state) = state_changes.recv() {
        tx_state.send(state).unwrap();
    }
}

