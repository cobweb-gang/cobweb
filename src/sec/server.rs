use spake2::{Ed25519Group, Identity, Password, SPAKE2};
use keybob::{Key, KeyType};
use tokio_core::net::UdpSocket;
use std::net::{SocketAddr, SocketAddrV4};
use std::str::FromStr;

pub fn handshake(client_id: &str, server_id: &str, sock: &UdpSocket, pass: &str) -> Key {
    let (spake, outbound_msg) = SPAKE2::<Ed25519Group>::start_b(
        &Password::new(pass.as_bytes()),
        &Identity::new(client_id.as_bytes()),
        &Identity::new(server_id.as_bytes()));

    let client_addr = SocketAddr::from(SocketAddrV4::from_str(client_id).unwrap());
    sock.connect(&client_addr).unwrap();
    sock.send(outbound_msg.as_slice()).unwrap();
    let inbound_msg: &mut [u8] = &mut [0u8];
    sock.recv(inbound_msg).unwrap();

    let key = spake.finish(&inbound_msg).unwrap();
    let key_pass = format!("{:?}", key);
    Key::from_pw(KeyType::Aes128, &key_pass, client_id)
}
