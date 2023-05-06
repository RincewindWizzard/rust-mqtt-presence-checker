use crate::config::{ApplicationConfig, ApplicationContext};

mod config;
mod args;

use anyhow::Context;
use rumqttc::{MqttOptions, AsyncClient, QoS};
use tokio::{task, time};
use std::time::Duration;
use std::error::Error;
use std::fmt::Display;
use crate::args::Args;
use clap::Parser;
use log::{debug, error};

fn setup_logging(args: &Args) -> Result<(), log::SetLoggerError> {
    stderrlog::new()
        .module(module_path!())
        .quiet(args.quiet)
        .verbosity(args.verbose as usize + 1) // show warnings and above
        .timestamp(stderrlog::Timestamp::Millisecond)
        .init()
}

async fn foo() {
    println!("Fooo!");
}

fn handle_error<V, E>(result: Result<V, E>) -> V
    where
        E: Display
{
    match result {
        Ok(value) => { value }
        Err(error) => {
            error!("An Error occured: {}", error);
            panic!()
        }
    }
}

async fn init() -> Result<ApplicationContext, anyhow::Error> {
    let args: Args = Args::parse();
    setup_logging(&args).expect("Failed to setup logging!");
    debug!("Setup logging!!");
    let context: Result<ApplicationContext, anyhow::Error> = ApplicationContext::from(args);

    context
}

#[tokio::main]
async fn main() {
    let context = handle_error(init().await);
    run(context).await;
}

async fn run(context: ApplicationContext) {
    debug!("Connecting to mqtt broker");
    let mut mqtt_options = MqttOptions::new(
        env!("CARGO_PKG_NAME"),
        context.config.broker.hostname,
        context.config.broker.port);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_credentials(context.config.broker.username, context.config.broker.password);
    debug!("Connecting to broker {:?}", mqtt_options);



    let (mut client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    client.subscribe("hello/rumqtt", QoS::AtMostOnce).await.unwrap();

    task::spawn(async move {
        for i in 0..100 {
            client.publish("hello/rumqtt", QoS::AtLeastOnce, false, format!("Meesage with id: {i}")).await.unwrap();
            time::sleep(Duration::from_millis(1000)).await;
        }
    });


    while let Ok(notification) = eventloop.poll().await {
        println!("Received = {:?}", notification);
    }
    debug!("looop");
}


