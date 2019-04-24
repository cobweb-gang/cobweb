use spake2::{Ed25519Group, Identity, Password, SPAKE2};
use keybob::{Key, KeyType};
use tokio_core::net::TcpStream;
use tokio::prelude::*;
use std::net::SocketAddr;

pub fn handshake(client_id: &str, server_addr: &SocketAddr, mut sock: &TcpStream, pass: &str) -> Result<Key, &'static str> {
    let (spake, outbound_msg) = SPAKE2::<Ed25519Group>::start_a(
        &Password::new(pass.as_bytes()),
        &Identity::new(client_id.as_bytes()),
        &Identity::new(format!("{}", server_addr).as_bytes())
        );

    sock.write_async(&mut outbound_msg.as_slice());

    let mut inbound_msg: &mut [u8] = &mut [0u8; 33];
    sock.read_async(&mut inbound_msg);

    let key = spake.finish(&inbound_msg).unwrap();
    let key_pass = format!("{:?}", key);

    Ok(Key::from_pw(KeyType::Aes128, &key_pass, client_id))
}
