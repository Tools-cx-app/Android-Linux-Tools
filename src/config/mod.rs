use std::{collections::HashMap, fs, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {
    pub user: String,
    pub shell: Shell,
    pub home: String,
    pub envs: Vec<(String, String)>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Shell {
    pub main: String,
    pub args: String,
}

impl Config {
    pub fn init(target: impl AsRef<Path>) -> Result<()> {
        let target = target.as_ref();
        let config_path = target.join("config.toml");
        let envs = vec![
            (
                "PATH".to_string(),
                "/usr/local/bin:/usr/bin:/bin".to_string(),
            ),
            ("TERM".to_string(), "xterm-256color".to_string()),
            ("SHELL".to_string(), "/bin/bash".to_string()),
            ("LANG".to_string(), "C.UTF-8".to_string()),
        ];
        let config = Self {
            user: "root".to_string(),
            home: "/root".to_string(),
            shell: Shell {
                main: "bash".to_string(),
                args: "-l".to_string(),
            },
            envs: envs,
        };

        if config_path.exists() {
            return Ok(());
        }

        let config = toml::to_string(&config)?;
        fs::write(config_path, config)?;
    }

    pub fn read_config(target: impl AsRef<Path>) -> Result<Self> {
        let target = target.as_ref();
        let file = fs::read_to_string(target.join("config.toml")).unwrap();
        let toml: Self = toml::from_str(file.as_str()).unwrap();
        Ok(toml)
    }
}
