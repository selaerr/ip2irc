#![warn(clippy::pedantic)]
use ::irc::client::data::Config;
use ::tun::AbstractDevice;
use bytes::FrameN;
use config::{into_octet_tuple, parse_config};
use flume::unbounded;
use irc::{init_irc_client, listen_irc, write_irc};
use macro_rules_attribute::apply;
use smol::future::race;
use smol_macros::{Executor, main};
use tun::{create_iface, listen_iface, write_iface};

mod bytes;
mod config;
mod irc;
mod tun;

static MTU: usize = 1500;
type Frame = FrameN<MTU>;

#[apply(main!)]
async fn main(ex: &Executor<'_>) {
    let conf = parse_config("ip2irc_config.toml").await.unwrap();
    println!("{conf:?}");

    let irc_config = Config {
        username: Some(conf.irc.username),
        nickname: Some(conf.irc.nickname),
        server: Some(conf.irc.server),
        channels: conf.irc.channels,
        port: Some(conf.irc.port),
        use_tls: Some(conf.irc.tls),
        ..Default::default()
    };

    let mut client = init_irc_client(irc_config).await.unwrap();
    let sender = client.sender();
    let stream = client
        .stream()
        .expect("Can't get a stream from the irc server");

    // the tunnel address is a /24 because this just defines peering!
    // in fact, all other subnets are larger than a /24,
    // under 10.69.0.0/16, and the way they're imported
    // into the kernel is via BGP!
    let dev = create_iface(
        into_octet_tuple(conf.tun.ip),
        into_octet_tuple(conf.tun.netmask),
        MTU.try_into().unwrap(),
    );
    println!(
        "created iface {}",
        dev.tun_name().expect("doesn't have a name??")
    );
    let (mut read_dev, mut write_dev) = dev.split();

    // channel for communicating data between threads
    let (tx_to_dev, rx_from_irc) = unbounded::<Frame>();
    let (tx_to_irc, rx_from_dev) = unbounded::<Frame>();

    let irc_listener_thread = ex.spawn(listen_irc::<Frame, MTU>(stream, tx_to_dev));
    let irc_writer_thread = ex.spawn(write_irc(
        sender,
        rx_from_dev,
        conf.irc.message_delay_millis,
        conf.irc.burst_messages,
    ));
    let tun_listener_thread =
        ex.spawn(async move { listen_iface::<Frame, MTU>(&mut read_dev, &tx_to_irc) });
    let tun_writer_thread = ex.spawn(async move { write_iface(&mut write_dev, &rx_from_irc) });

    // await two futures at once
    let tun = race(tun_listener_thread, tun_writer_thread);
    let irc = race(irc_listener_thread, irc_writer_thread);
    race(tun, irc).await;
}
