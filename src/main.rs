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

    let mut heartbeats = vec![];
    for probe in context.config.ping.hosts {
        heartbeats.push(ping(probe.host.clone(), probe.get_interval()))
    }

    let (minuterie_tx, state_changes) = Minuterie::new(context.config.minuterie.get_timeout());

    merge_channels(heartbeats, minuterie_tx);




    let (mut client, mut connection) = Client::new(MqttOptions::from(&context.config.mqtt), 10);

    thread::spawn(move || {
        for (i, notification) in connection.iter().enumerate() {
            trace!("Notification = {:?}", notification);
        }
    });

    while let Ok(state) = state_changes.recv() {
        let topic = &context.config.mqtt.topic;
        let presence = state.state.to_string();
        debug!("Announcing presence ({presence}) to \"{topic}\"");
        client.publish(topic, QoS::AtLeastOnce, false, presence).unwrap();
    }

}

