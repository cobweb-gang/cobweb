extern crate cobweb;
extern crate tokio_core;
extern crate futures;
extern crate tun_tap;
extern crate keybob;

use std::io::Result;
use tokio_core::reactor::Core;
use tokio_core::net::UdpSocket;
use cobweb::vpn::{EncryptedTun, UdpVecCodec};
use cobweb::sec::{En, De};
use tun_tap::async::Async;
use keybob::{Key, KeyType};
use futures::stream::{SplitStream, SplitSink};
use futures::sink::With;
use futures::stream::Map;
use futures::prelude::*;

fn main() {
    let loc_addr = "127.0.0.1:1337".parse().unwrap();
    let rem_addr = "127.0.0.1:1338".parse().unwrap();
    let key = Key::from_pw(KeyType::Aes128, "test", "cobweb");
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let tun = EncryptedTun::<With<SplitSink<Async>, Vec<u8>, De, Result<Vec<u8>>>, Map<SplitStream<Async>, En>>::new(&key, &handle);
    let (tun_sink, tun_stream) = tun.split();

    let sock = UdpSocket::bind(&loc_addr, &handle).unwrap();
    let (udp_sink, udp_stream) = sock.framed(UdpVecCodec::new(rem_addr))
    	.split();

    let sender = tun_stream.forward(udp_sink);
    let receiver = udp_stream.forward(tun_sink);
    core.run(sender.join(receiver))
        .unwrap();
}
