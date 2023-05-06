use std::fs;
use std::path::PathBuf;
use anyhow::anyhow;
use directories::ProjectDirs;
use log::debug;
use serde::de::Error;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApplicationConfig {
    #[serde(skip)]
    #[serde(default = "ApplicationConfig::project_dirs_unwrapped")]
    pub project_dirs: ProjectDirs,
    pub broker: MqttBroker,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MqttBroker {
    pub hostname: String,
    pub username: String,
    pub password: String,
}


impl ApplicationConfig {
    pub fn load_config() -> Result<ApplicationConfig, anyhow::Error> {
        let project_dirs: ProjectDirs = ApplicationConfig::project_dirs()?;
        let result: ApplicationConfig = confy::load_path(project_dirs.config_dir())?;
        Ok(result)
    }

    fn project_dirs_unwrapped() -> ProjectDirs {
        ApplicationConfig::project_dirs().unwrap()
    }

    /// In debug mode create project data in local path
    #[cfg(debug_assertions)]
    fn project_dirs() -> Result<ProjectDirs, anyhow::Error> {
        let data_path =
            fs::canonicalize(
                std::env::current_exe()?
                    .parent()
                    .ok_or(anyhow!("Could not get parent path of exe!"))?
                    .join("data")
            )?;

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
            /// project_dirs has to be present at this point!
            project_dirs: ApplicationConfig::project_dirs().unwrap(),
            broker: MqttBroker {
                hostname: "mqtt.example.org".to_string(),
                username: "username".to_string(),
                password: "password".to_string(),
            },
        }
    }
}