use std::io;
use tokio_core::net::{TcpListener};
use tokio_core::net::{UdpCodec, UdpSocket};
use tokio_core::reactor::Core;
use tokio_codec::{Encoder, Decoder};
use tokio_io::AsyncRead;
use tun_tap::{Iface, Mode};
use tun_tap::async::Async;
use std::process::Command;
use std::net::SocketAddr;
use std::io::Result;
use bytes::BytesMut;
use futures::prelude::*;

struct TcpVecCodec;

impl Decoder for TcpVecCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn decode (&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
        Ok(Some(buf.iter().cloned().collect()))
    }
}

impl Encoder for TcpVecCodec {
    type Item = Vec<u8>;
    type Error = io::Error;

    fn encode (&mut self, item: Self::Item, dst: &mut BytesMut) -> io::Result<()> {
        dst.extend_from_slice(item.as_slice());

        Ok(())
    }
}

struct UdpVecCodec(SocketAddr);

impl UdpCodec for UdpVecCodec {
	type In = Vec<u8>;
	type Out = Vec<u8>;
	fn decode(&mut self, _src: &SocketAddr, buf: &[u8]) -> Result<Self::In> {
		Ok(buf.to_owned())
	}
	fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> SocketAddr {
		buf.extend(&msg);
		self.0
	}
}

fn cmd(cmd: &str, args: &[&str]) {
    let ecode = Command::new("ip")
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(ecode.success(), "Failed to execte {}", cmd);
}

pub fn start_tun_tcp() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let loc_address = "127.0.0.1:6969".parse().unwrap();
    let listener = TcpListener::bind(&loc_address, &handle)
        .unwrap();
    let server = listener.incoming().for_each(|(sock, _)| {
        println!("Ayy!! We got a new connection");
        println!("{:?}", sock);
		let tun = Iface::new("vpn%d", Mode::Tun)
			.unwrap();
		eprintln!("Iface: {:?}", tun);
		cmd("ip", &["addr", "add", "dev", tun.name(), "10.107.1.3/24"]);
		cmd("ip", &["link", "set", "up", "dev", tun.name()]);
        let (sender, receiver) = sock.framed(TcpVecCodec)
            .split();
        let (sink, stream) = Async::new(tun, &handle)
            .unwrap()
            .split();
        let reader = stream.forward(sender);
        let writer = receiver.forward(sink);
        
        Ok(())
    });

    core.run(server).unwrap();
}

pub fn start_tun_udp() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let loc_address = "127.0.0.1:6969".parse().unwrap();
    let rem_address = "0.0.0.0:11443".parse().unwrap();
    let socket = UdpSocket::bind(&loc_address, &handle)
        .unwrap();
    println!("Listening on port 6969");
    let (sender, receiver) = socket.framed(UdpVecCodec(rem_address))
        .split();
    let tun = Iface::new("vpn%d", Mode::Tun)
        .unwrap();
    eprintln!("Iface: {:?}", tun);
    cmd("ip", &["addr", "add", "dev", tun.name(), "10.107.1.3/24"]);
    cmd("ip", &["link", "set", "up", "dev", tun.name()]);
    let (sink, stream) = Async::new(tun, &handle)
        .unwrap()
        .split();
    let reader = stream.forward(sender);
    let writer = receiver.forward(sink);
    core.run(reader.join(writer))
        .unwrap();
}
