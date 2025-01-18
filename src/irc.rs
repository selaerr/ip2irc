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
            if let Command::PRIVMSG(chan, msg) = message.command {
                println!("received {msg:?} from {chan:?}");
                let read = BASE64_STANDARD
                    .decode_slice(msg.as_bytes(), buf.as_mut_slice())
                    .expect("failed to decode message");
                tx.send(IntoBytes::from_buf(buf, read))
                    .expect("failed to transmit to channel");
            }
        }
    }));
}

pub async fn write_irc<T: AsBytes>(sender: irc::client::Sender, recv: Receiver<T>) {
    while let Ok(data) = recv.recv() {
        let enc = BASE64_STANDARD.encode(data.as_slice());
        sender
            .send_privmsg("#test", enc)
            .expect("failed to send irc message");
    }
}
