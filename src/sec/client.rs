use spake2::{Ed25519Group, Identity, Password, SPAKE2};
use keybob::{Key, KeyType};
use tokio_core::net::TcpStream;
use std::net::SocketAddr;
use std::io::{Read, Write};

pub fn handshake(client_id: &str, server_addr: &SocketAddr, mut sock: &TcpStream, pass: &str) -> Result<Key, &'static str> {
    let (spake, outbound_msg) = SPAKE2::<Ed25519Group>::start_a(
        &Password::new(pass.as_bytes()),
        &Identity::new(client_id.as_bytes()),
        &Identity::new(format!("{}", server_addr).as_bytes())
        );

    sock.write(outbound_msg.as_slice()).unwrap();

    let inbound_msg: &mut [u8] = &mut [0u8];
    sock.read(inbound_msg).unwrap();

    let key = spake.finish(&inbound_msg).unwrap();
    let key_pass = format!("{:?}", key);

    Ok(Key::from_pw(KeyType::Aes128, &key_pass, client_id))
}
