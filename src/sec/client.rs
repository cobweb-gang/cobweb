use spake2::{Ed25519Group, Identity, Password, SPAKE2};
use keybob::{Key, KeyType};
use tokio_core::net::UdpSocket;
use std::net::SocketAddr;

pub fn handshake(client_id: &str, server_addr: &SocketAddr, sock: &UdpSocket, pass: &str) -> Key {
    let (spake, outbound_msg) = SPAKE2::<Ed25519Group>::start_a(
        &Password::new(pass.as_bytes()),
        &Identity::new(client_id.as_bytes()),
        &Identity::new(format!("{}", server_addr).as_bytes())
        );

    sock.connect(server_addr).unwrap();
    sock.send(outbound_msg.as_slice()).unwrap();
    let inbound_msg: &mut [u8] = &mut [0u8];
    sock.recv(inbound_msg).unwrap();

    let key = spake.finish(&inbound_msg).unwrap();
    let key_pass = format!("{:?}", key);
    Key::from_pw(KeyType::Aes128, &key_pass, client_id)
}
