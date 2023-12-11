
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::anyhow;
use clap::Parser;
use directories::ProjectDirs;
use log::debug;
use rumqttc::MqttOptions;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(debug_assertions)]
const DEBUG_APPLICATION_CONTEXT_PATH: &str = "application_context";

#[cfg(debug_assertions)]
use std::fs;

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct ApplicationContext {
    pub(crate) project_dirs: ProjectDirs,
    pub(crate) args: Args,
    pub(crate) launch: Instant,
    pub(crate) config: ApplicationConfig,
}

impl ApplicationContext {
    /// In debug mode create project data in local path
    #[cfg(debug_assertions)]
    pub fn project_dirs() -> Option<ProjectDirs> {
        let exe_path = std::env::current_exe().unwrap();
        let mut data_path = PathBuf::from(exe_path.parent()?);
        data_path.push(DEBUG_APPLICATION_CONTEXT_PATH);

        debug!("Create path: {}", data_path.display());
        fs::create_dir_all(&data_path).expect("Data Path can not be created!");

        let data_path = fs::canonicalize(&data_path).unwrap_or_else(|_| panic!("Data Path '{}' does not exist!", data_path.display()));


        let project_dirs = ProjectDirs::from_path(data_path)?;

        debug!("{project_dirs:?}");
        Some(project_dirs)
    }

    /// In production mode create project data in the correct paths
    #[cfg(not(debug_assertions))]
    pub fn project_dirs() -> Option<ProjectDirs> {
        let project_dirs = ProjectDirs::from(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_HOMEPAGE"),
            env!("CARGO_PKG_NAME"))?;
        debug!("{project_dirs:?}");
        Some(project_dirs)
    }
}


impl ApplicationContext {
    pub(crate) fn construct() -> anyhow::Result<ApplicationContext> {
        let args = Args::parse();
        let project_dirs = ApplicationContext::project_dirs().ok_or(anyhow!("Could not load project dirs!"))?;

        let config = confy::load_path(if let Some(path) = args.config {
            PathBuf::from(path)
        } else {
            ApplicationContext::config_file_path(&project_dirs)
        })?;


        Ok(ApplicationContext {
            project_dirs,
            args: Args::parse(),
            launch: Instant::now(),
            config,
        })
    }


    fn config_file_path(project_dirs: &ProjectDirs) -> PathBuf {
        let mut config_file_path = PathBuf::from(project_dirs.config_dir());
        config_file_path.push(format!("{}.toml", env!("CARGO_PKG_NAME")));
        config_file_path
    }
}


/// Presence checker for your smart home
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub(crate) verbose: u8,

    /// Path to config file
    #[arg(short, long)]
    config: Option<String>,

    /// no stdout printing
    #[arg(short, long)]
    pub(crate) quiet: bool,

}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct ApplicationConfig {
    pub(crate) minuterie: MinuterieConfig,
    pub(crate) mqtt: Mqtt,
    pub(crate) ping: PingConfig,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        ApplicationConfig {
            ping: PingConfig {
                hosts: vec![
                    PingProbe {
                        host: "fritz.box".to_string(),
                        interval: 1,
                    },
                    PingProbe {
                        host: "google.de".to_string(),
                        interval: 1,
                    },
                ]
            },
            minuterie: MinuterieConfig { timeout: 60 },
            mqtt: Mqtt {
                host: "127.0.0.1".to_string(),
                username: "username".to_string(),
                password: "password".to_string(),
                port: 1883,
                publish_topic: "mqtt-presence-checker/home/".to_string(),
                heartbeat_topic: "presence-checker1/home/heartbeat".to_string(),
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct MinuterieConfig {
    timeout: u64,
}

impl MinuterieConfig {
    pub(crate) fn get_timeout(&self) -> Duration {
        Duration::from_secs(self.timeout)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct PingConfig {
    pub(crate) hosts: Vec<PingProbe>,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct PingProbe {
    pub(crate) host: String,
    interval: u64,
}

impl PingProbe {
    pub(crate) fn get_interval(&self) -> Duration {
        Duration::from_secs(self.interval)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct Mqtt {
    pub(crate) host: String,
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) port: u16,
    pub(crate) publish_topic: String,
    pub(crate) heartbeat_topic: String,
}

impl From<&Mqtt> for MqttOptions {
    fn from(config: &Mqtt) -> Self {
        let mut mqttoptions = MqttOptions::new(
            format!("{}-{}", env!("CARGO_PKG_NAME"), Uuid::new_v4()),
            &config.host,
            config.port,
        );
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        mqttoptions.set_credentials(&config.username, &config.password);
        mqttoptions
    }
}

