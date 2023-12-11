use std::io::Bytes;
use std::io::ErrorKind::ConnectionRefused;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SendError};
use std::thread;
use std::time::Duration;
use anyhow::anyhow;
use log::{debug, error, trace, warn};
use rumqttc::{Client, ConnectionError, Event, MqttOptions, QoS};
use rumqttc::ConnectionError::Io;
use rumqttc::Packet::Publish;
use crate::args::ApplicationContext;
use crate::minuterie::{Heartbeat, StateChange, State};


/// Connects to a mqtt server and listens on the heartbeat_topic.
/// Every Statechange sent will be published to the publish_topic.
/// Every heartbeat received will be send to the heartbeat sender.
pub fn mqtt_connect(mqtt_options: MqttOptions, heartbeat_topic: &str, publish_topic: &str) -> (Sender<StateChange>, Receiver<Heartbeat>) {
    let (mut client, mut connection) = Client::new(mqtt_options, 10);

    let (tx_state_change, rx) = mpsc::channel::<StateChange>();
    let (tx, rx_heartbeat) = mpsc::channel();

    if let Ok(_) = client.subscribe(heartbeat_topic, QoS::AtMostOnce) {
        thread::spawn(move || {
            for (i, notification) in connection.iter().enumerate() {
                trace!("MQTT notification: {:?}", notification);
                if notification.is_err() {
                    if let Err(Io(err)) = &notification {
                        if err.kind() == ConnectionRefused {
                            warn!("Error in connection with mqtt: {}", err.to_string());
                            thread::sleep(Duration::from_secs(10));
                        }
                    }
                }

                if let Some(heartbeat) = on_incoming_message(notification) {
                    tx.send(heartbeat)?;
                }
            }
            Ok::<(), SendError<Heartbeat>>(())
        });
    }

    let publish_topic = String::from(publish_topic);
    thread::spawn(move || {
        while let Ok(state_change) = rx.recv() {
            let payload = serde_json::to_string(&state_change);
            if let Ok(payload) = payload {
                debug!("Published {}", state_change.state);
                client.publish(&publish_topic, QoS::AtLeastOnce, false, payload).unwrap();
            } else {
                error!("Error while publishing state: {:?}", payload);
            }
        }
    });
    (tx_state_change, rx_heartbeat)
}

fn on_incoming_message(notification: Result<Event, ConnectionError>) -> Option<Heartbeat> {
    if let Ok(Event::Incoming(ref incomin)) = notification {
        if let Publish(data) = incomin {
            if let Ok(payload) = std::str::from_utf8(data.payload.as_ref()) {
                if is_trueish(payload) {
                    return Some(Heartbeat::default());
                }
            }
        }
    }
    None
}

/// Returns if a value looks like it is meant to be true
fn is_trueish(data: &str) -> bool {
    let trueish_values = [
        "1", "true", "True", "On"
    ];
    trueish_values.iter().any(|x| x == &data)
}
