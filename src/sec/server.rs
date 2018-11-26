use spake2::{Ed25519Group, Identity, Password, SPAKE2};
use keybob::{Key, KeyType};
use tokio_core::net::UdpSocket;

pub fn handshake(client_id: &str, server_id: &str, pass: &str, sock: &UdpSocket) -> Key {
    let (spake, outbound_msg) = SPAKE2::<Ed25519Group>::start_b(
        &Password::new(pass.as_bytes()),
        &Identity::new(client_id.as_bytes()),
        &Identity::new(server_id.as_bytes()));

    sock.send(outbound_msg.as_slice()).unwrap();
    let inbound_msg: &mut [u8] = &mut [0u8];
    sock.recv(inbound_msg).unwrap();

    let key = spake.finish(&inbound_msg).unwrap();
    let key_pass = format!("{:?}", key);
    Key::from_pw(KeyType::Aes128, &key_pass, server_id)
}
