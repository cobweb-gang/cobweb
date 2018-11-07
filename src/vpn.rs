use tokio_core::net::{UdpCodec, UdpSocket, UdpFramed};
use tokio_core::reactor::Handle;
use tun_tap::{Iface, Mode};
use tun_tap::async::Async;
use std::process::Command;
use std::net::SocketAddr;
use std::io::Result;
use futures::prelude::*;
use futures::stream::{SplitSink, SplitStream};

fn cmd(cmd: &str, args: &[&str]) {
    let ecode = Command::new("ip")
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(ecode.success(), "Failed to execte {}", cmd);
}

pub struct UdpVecCodec(SocketAddr);

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

pub struct VpnSocket<T: Sink, U: Stream, V: Sink, W: Stream> {
    tun_sink: T,
    tun_stream: U,
    udp_sink: V,
    udp_stream: W,
}

impl<T, U, V, W> VpnSocket<T, U, V, W> 
where T: Sink,
      U: Stream,
      T::SinkItem: From<Vec<u8>>,
      U::Item: std::convert::AsRef<[u8]>,
      U::Error: std::fmt::Debug,
      V: Sink,
      W: Stream,
      V::SinkItem: From<Vec<u8>>,
      W::Item: std::convert::AsRef<[u8]>,
      W::Error: std::fmt::Debug,
{

    pub fn connect(loc_addr: &SocketAddr, rem_addr: &SocketAddr, handle: &Handle) -> Result<VpnSocket<SplitSink<Async>, SplitStream<Async>, SplitSink<UdpFramed<UdpVecCodec>>, SplitStream<UdpFramed<UdpVecCodec>>>> {
        let socket = UdpSocket::bind(loc_addr, handle).unwrap();
        let (sender, receiver) = socket.framed(UdpVecCodec(*rem_addr))
            .split();
        let tun = Iface::new("vpn%d", Mode::Tun)
            .unwrap();
        cmd("ip", &["addr", "add", "dev", tun.name(), "10.107.1.3/24"]);
        cmd("ip", &["link", "set", "up", "dev", tun.name()]);
        let (sink, stream) = Async::new(tun, &handle)
            .unwrap()
            .split();
        Ok(VpnSocket {
            tun_sink: sink,
            tun_stream: stream,
            udp_sink: sender,
            udp_stream: receiver
        })
    }

    pub fn get_tun(self) -> Result<(T, U)> {
        Ok((self.tun_sink, self.tun_stream))
    }

    pub fn get_udp(self) -> Result<(V, W)> {
        Ok((self.udp_sink, self.udp_stream))
    }

    pub fn get_all(self) -> Result<((T, U), (V, W))> {
        Ok(((self.tun_sink, self.tun_stream), (self.udp_sink, self.udp_stream)))
    }
}
