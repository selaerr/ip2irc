use std::net::Ipv4Addr;

use serde::Deserialize;
use smol::{fs::File, io::AsyncReadExt};
use toml::from_str;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub irc: IrcConfiguration,
    pub tun: TunConfiguration,
}

#[derive(Deserialize, Debug)]
pub struct IrcConfiguration {
    pub nickname: String,
    pub username: String,
    pub server: String,
    pub channels: Vec<String>,
    pub port: u16,
    pub tls: bool,
}

#[derive(Deserialize, Debug)]
pub struct TunConfiguration {
    pub ip: Ipv4Addr,
    pub netmask: Ipv4Addr,
}

pub async fn parse_config(file: &str) -> anyhow::Result<Configuration> {
    let mut open = File::open(file).await?;
    let mut contents = String::new();
    let _ = open.read_to_string(&mut contents).await?;
    Ok(from_str(&contents)?)
}

pub fn into_octet_tuple(ip: Ipv4Addr) -> (u8, u8, u8, u8) {
    let octets = ip.octets();
    (octets[0], octets[1], octets[2], octets[3])
}
