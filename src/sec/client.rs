use spake2::{Ed25519Group, Identity, Password, SPAKE2};
use keybob::{Key, KeyType};
use std::net::UdpSocket;
use futures::prelude::*;
use futures_retry::{FutureRetry, RetryPolicy};
use std::net::SocketAddr;
use std::time::Duration;
use std::io::Error;

pub fn handshake(client_id: &str, server_addr: &SocketAddr, sock: &UdpSocket, pass: &str) -> Result<Key, &'static str> {
    let (spake, outbound_msg) = SPAKE2::<Ed25519Group>::start_a(
        &Password::new(pass.as_bytes()),
        &Identity::new(client_id.as_bytes()),
        &Identity::new(format!("{}", server_addr).as_bytes())
        );

    FutureRetry::new(|| sock.connect(server_addr).into_future(), |_: Error| RetryPolicy::<Error>::WaitRetry(Duration::new(5, 0)));
    FutureRetry::new(|| sock.send(outbound_msg.as_slice()).into_future(), |_: Error| RetryPolicy::<Error>::WaitRetry(Duration::new(5, 0)));

    let inbound_msg: &mut [u8] = &mut [0u8];
    FutureRetry::new(|| sock.recv(inbound_msg).into_future(), |_: Error| RetryPolicy::<Error>::WaitRetry(Duration::new(5, 0)));

    let key = spake.finish(&inbound_msg).unwrap();
    let key_pass = format!("{:?}", key);

    Ok(Key::from_pw(KeyType::Aes128, &key_pass, client_id))
}
