use std::io::Bytes;
use std::sync::mpsc;
use std::thread;
use log::{debug, trace};
use rumqttc::{Client, ConnectionError, Event, MqttOptions, QoS};
use rumqttc::Packet::Publish;
use crate::args::ApplicationContext;
use crate::minuterie::Heartbeat;

pub fn foo(context: &ApplicationContext) {
    let (mut client, mut connection) = Client::new(MqttOptions::from(&context.config.mqtt), 10);

    let heartbeat_topic = "presence-checker1/home/heartbeat";

    let (tx, rx) = mpsc::channel();
    client.subscribe(heartbeat_topic, QoS::AtMostOnce).unwrap();
    let handle = thread::spawn(move || {
        for (i, notification) in connection.iter().enumerate() {
            if let Ok(Event::Incoming(ref incomin)) = notification {
                if let Publish(data) = incomin {
                    debug!("{}: {:?}", data.topic, data.payload);
                    if let Ok(payload) =  std::str::from_utf8(data.payload.as_ref()) {
                        if is_trueish(payload) {
                            tx.send(Heartbeat::default()).unwrap();
                        }
                    }
                }
            }
            trace!("Notification = {:?}", notification);
        }
    });

    while let Ok(state) = rx.recv() {
        debug!("Heartbeat");
    }
    // while let Ok(state) = state_changes.recv() {
    //     let topic = &context.config.mqtt.topic;
    //     let presence = state.state.to_string();
    //     debug!("Announcing presence ({presence}) to \"{topic}\"");
    //     client.publish(topic, QoS::AtLeastOnce, false, presence).unwrap();
    // }
}

fn is_trueish(data: &str) -> bool {
    let trueish_values = [
        "1", "true", "True", "On"
    ];
    trueish_values.iter().any(|x| x == &data)
}
