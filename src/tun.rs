use std::io::{Read, Write};

use flume::{Receiver, Sender};
use tun::{Device, Reader, Writer};

use crate::bytes::{AsBytes, IntoBytes};

/// Creates a tunnel interface with the specified IP address and netmask.
pub fn create_iface(ip_addr: (u8, u8, u8, u8), netmask: (u8, u8, u8, u8), mtu: u16) -> Device {
    let mut config = tun::Configuration::default();
    config.address(ip_addr).netmask(netmask).mtu(mtu).up();

    tun::create(&config).expect("failed to create tunnel")
}

/// Listens on the given device.
/// `N`: buffer size
/// `dev`: the tunnel device (a reader to it)
/// `tx`: mpsc channel where to send data, once read
pub fn listen_iface<T: IntoBytes<N>, const N: usize>(dev: &mut Reader, tx: &Sender<T>) {
    let mut buf = [0; N];
    loop {
        // todo: use async read? yes
        let amount = dev.read(&mut buf).expect("failed to read into buf");
        println!("{:x?}", &buf[0..amount]);
        tx.send(IntoBytes::from_buf(buf, amount))
            .expect("channel is dead on write!");
    }
}

/// Writes to given device.
/// `dev`: the tunnel device (a writer to it)
/// `rx`: mpsc channel from which to read data
pub fn write_iface<T: AsBytes>(dev: &mut Writer, rx: &Receiver<T>) {
    while let Ok(bytes) = rx.recv() {
        // todo: use async write?
        dev.write_all(bytes.as_slice()).expect("failed to write!");
    }
}
