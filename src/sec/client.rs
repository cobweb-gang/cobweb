use spake2::{Ed25519Group, Identity, Password, SPAKE2};
use keybob::{Key, KeyType};
use tokio_core::net::TcpStream;
use tokio_io::io::{read, write_all};
use futures::prelude::*;
use std::net::SocketAddr;

pub fn handshake(client_id: &str, server_addr: &SocketAddr, sock: &TcpStream, pass: &str) -> Result<Key, &'static str> {
    let (spake, outbound_msg) = SPAKE2::<Ed25519Group>::start_a(
        &Password::new(pass.as_bytes()),
        &Identity::new(client_id.as_bytes()),
        &Identity::new(format!("{}", server_addr).as_bytes())
        );

    write_all(sock, &outbound_msg).wait().unwrap();

    let mut inbound_msg: &mut [u8] = &mut [0u8; 33];
    read(sock, &mut inbound_msg).wait().unwrap();

    let key = spake.finish(&inbound_msg).unwrap();
    let key_pass = format!("{:?}", key);

    Ok(Key::from_pw(KeyType::Aes128, &key_pass, client_id))
}
