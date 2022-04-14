//! Configuration loading

use anyhow::Result;
use std::{fs::OpenOptions, io::Read, net::SocketAddrV4, path::Path};
use yaml_rust::{Yaml, YamlLoader};

pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Conf> {
    let mut yaml = String::new();
    OpenOptions::new()
        .read(true)
        .open(path)?
        .read_to_string(&mut yaml)?;
    let raw_cfg = YamlLoader::load_from_str(&yaml)?[0]
        .clone()
        .into_hash()
        .unwrap();
    let bot_addr = raw_cfg
        .get(&Yaml::String("bot_addr".into()))
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
        .parse::<SocketAddrV4>()?;
    let controller_port = usize::try_from(
        raw_cfg
            .get(&Yaml::String("controller_port".into()))
            .unwrap()
            .as_i64()
            .unwrap(),
    )?;

    Ok(Conf {
        bot_addr,
        controller_port,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conf {
    pub bot_addr: SocketAddrV4,
    pub controller_port: usize,
}
