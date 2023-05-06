use std::fs;

use anyhow::anyhow;
use directories::ProjectDirs;
use log::debug;

use serde_derive::{Deserialize, Serialize};
use crate::args::Args;


#[derive(Debug, Clone)]
pub struct ApplicationContext {
    pub project_dirs: ProjectDirs,
    pub args: Args,
    pub config: ApplicationConfig,
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApplicationConfig {
    pub broker: MqttBroker,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MqttBroker {
    pub hostname: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}


impl ApplicationContext {
    pub fn from(args: Args) -> Result<ApplicationContext, anyhow::Error> {
        let project_dirs = ApplicationContext::project_dirs()?;
        let config = ApplicationContext::load_config(&project_dirs)?;
        Ok(ApplicationContext {
            project_dirs,
            args,
            config,
        })
    }

    fn load_config(project_dirs: &ProjectDirs) -> Result<ApplicationConfig, anyhow::Error> {
        let config_dir = project_dirs.config_dir().join("main.conf");
        debug!("config dir {}", config_dir.display());
        let result = confy::load_path(config_dir)?;
        Ok(result)
    }

    /// In debug mode create project data in local path
    #[cfg(debug_assertions)]
    fn project_dirs() -> Result<ProjectDirs, anyhow::Error> {
        let data_path =
            std::env::current_exe()?
                .parent()
                .ok_or(anyhow!("Could not get parent path of exe!"))?
                .join("data");

        let data_path = fs::canonicalize(data_path)?;

        debug!("Create path: {}", data_path.display());
        fs::create_dir_all(&data_path)?;

        let project_dirs = ProjectDirs::from_path(data_path)
            .ok_or(anyhow!("Could not get ProjectDirs from project path!"))?;
        debug!("{project_dirs:?}");

        Ok(project_dirs)
    }

    /// In production mode create project data in the correct paths
    #[cfg(not(debug_assertions))]
    fn project_dirs() -> Option<ProjectDirs> {
        let project_dirs = ProjectDirs::from(
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_HOMEPAGE"),
            env!("CARGO_PKG_NAME"))?;
        debug!("{project_dirs:?}");
        Some(project_dirs)
    }
}


impl Default for ApplicationConfig {
    fn default() -> ApplicationConfig {
        debug!("Created a new configuration from default.");
        ApplicationConfig {
            broker: MqttBroker {
                hostname: "mqtt.example.org".to_string(),
                port: 1883,
                username: "username".to_string(),
                password: "password".to_string(),
            },
        }
    }
}