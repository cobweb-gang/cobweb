use spake2::{Ed25519Group, Identity, Password, SPAKE2};
use keybob::{Key, KeyType};
use tokio_core::net::UdpSocket;
use std::net::SocketAddr;

pub fn handshake(client_addr: &SocketAddr, server_id: &str, sock: &UdpSocket, pass: &str) -> Result<Key, String> {
    let mut error = "";
    let (spake, outbound_msg) = SPAKE2::<Ed25519Group>::start_b(
        &Password::new(pass.as_bytes()),
        &Identity::new(format!("{}", client_addr).as_bytes()),
        &Identity::new(server_id.as_bytes()),
        );

    sock.connect(client_addr).unwrap_or_else(|_| {
        error = "ERROR: Unable to connect to server";
    });
    sock.send(outbound_msg.as_slice()).unwrap();
    let inbound_msg: &mut [u8] = &mut [0u8];
    sock.recv(inbound_msg).unwrap();

    let key = spake.finish(&inbound_msg).unwrap();
    let key_pass = format!("{:?}", key);

    let result = match error {
        "ERROR: Unable to connect to server" => Err(String::from(error)),
        _ => Ok(Key::from_pw(KeyType::Aes128, &key_pass, server_id)),
    };

    result
}
