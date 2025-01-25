use std::time::{Duration, Instant};

use async_compat::Compat;
use base64::prelude::*;
use flume::{Receiver, Sender};
use irc::{
    client::{Client, ClientStream, data::Config},
    proto::Command,
};
use smol::stream::StreamExt;

use crate::bytes::{AsBytes, IntoBytes};

pub async fn init_irc_client(config: Config) -> anyhow::Result<Client> {
    // todo: move config into main
    Compat::new(async {
        let client = Client::from_config(config)
            .await
            .expect("failed to initialize irc client");

        client.identify().expect("Failed to identify");
        Ok(client)
    })
    .await
}

pub async fn listen_irc<T: IntoBytes<N>, const N: usize>(mut stream: ClientStream, tx: Sender<T>) {
    let mut buf = [0; N];
    smol::future::block_on(Compat::new(async {
        while let Some(message) = stream.next().await.transpose().expect("damn wtf happened") {
            if let Command::PRIVMSG(_chan, msg) = message.command {
                // println!("received {msg:?} from {chan:?}");
                let read = BASE64_STANDARD.decode_slice(msg.as_bytes(), buf.as_mut_slice());
                // just ignore messages that aren't ok
                if read.is_err() {
                    continue;
                }
                tx.send(IntoBytes::from_buf(buf, read.unwrap()))
                    .expect("failed to transmit to channel");
            }
        }
    }));
}

pub async fn write_irc<T: AsBytes>(sender: irc::client::Sender, recv: Receiver<T>) {
    // time between messages
    // todo: make this configurable
    let sub = Duration::from_millis(100);
    // burstable messages
    let burst = 10 * sub;
    // time at which delay is over
    let mut time = Instant::now();
    loop {
        if let Ok(data) = recv.recv() {
            let msg = BASE64_STANDARD.encode(data.as_slice());
            // check if delay has elapsed. if so, will return zero.
            let mut delay = time.saturating_duration_since(Instant::now());
            // subtract bursting time
            delay = delay.saturating_sub(burst);
            // if the delay has elapsed:
            if delay.is_zero() {
                // on every message sent, add `sub` to the delay timing
                time = std::cmp::max(time, Instant::now()) + sub;
                sender
                    .send_privmsg("#test", msg)
                    .expect("failed to send irc msg");
            } else {
                drop(msg);
            }
        }
    }
}
