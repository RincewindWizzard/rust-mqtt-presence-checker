use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use anyhow::anyhow;
use clap::Parser;
use confy::ConfyError;
use directories::ProjectDirs;
use log::{debug, error};
use serde::{Deserialize, Serialize};

const DEBUG_APPLICATION_CONTEXT_PATH: &str = "application_context";
const CONFIG_FILE_NAME: &str = "main.toml";

#[derive(Debug)]
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
        data_path.push("application_context");

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
        let project_dirs = ApplicationContext::project_dirs().ok_or(anyhow!("Could not load project dirs!"))?;
        let config = confy::load_path(crate::args::ApplicationContext::config_file_path(&project_dirs))?;

        Ok(ApplicationContext {
            project_dirs,
            args: Args::parse(),
            launch: Instant::now(),
            config,
        })
    }


    fn config_file_path(project_dirs: &ProjectDirs) -> PathBuf {
        let mut config_file_path = PathBuf::from(project_dirs.config_dir());
        config_file_path.push(CONFIG_FILE_NAME);
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

    #[arg(short, long)]
    config_file: Option<String>,

    /// no stdout printing
    #[arg(short, long)]
    pub(crate) quiet: bool,

}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct ApplicationConfig {
    minuterie: MinuterieConfig,
    mqtt: Mqtt,
    ping: PingConfig,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        ApplicationConfig {
            ping: PingConfig {
                hosts: vec![
                    PingProbe {
                        host: "fritz.box".to_string(),
                        interval: 1000,
                    },
                    PingProbe {
                        host: "google.de".to_string(),
                        interval: 1000,
                    },
                ]
            },
            minuterie: MinuterieConfig { timeout: 1000 },
            mqtt: Mqtt {
                host: "127.0.0.1".to_string(),
                username: "username".to_string(),
                password: "password".to_string(),
                topic: "mqtt-presence-checker/home/".to_string(),
            },
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct MinuterieConfig {
    timeout: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct PingConfig {
    hosts: Vec<PingProbe>,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct PingProbe {
    host: String,
    interval: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct Mqtt {
    host: String,
    username: String,
    password: String,
    topic: String,
}

