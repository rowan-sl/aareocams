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
    let addr = raw_cfg
        .get(&Yaml::String("addr".into()))
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
        .parse::<SocketAddrV4>()?;

    Ok(Conf { addr })
}

#[derive(Debug, Clone, PartialEq)]
pub struct Conf {
    pub addr: SocketAddrV4,
}
