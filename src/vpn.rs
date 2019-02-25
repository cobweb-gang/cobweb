use sec::en::{En, De};
use sec::client::handshake;
use std::io::Result;
use keybob::{Key, KeyType};
use tun_tap::{Iface, Mode};
use tun_tap::async::Async;
use mac_utun::get_utun;
use std::process::Command;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::{UdpSocket, UdpCodec};
use futures::prelude::*;
use futures::stream::{SplitSink, SplitStream};
use futures::sink::With;
use futures::stream::Map;
use std::result::Result as DualResult;
use std::net::UdpSocket as SyncUdpSocket;

#[cfg(target_os = "linux")]
pub fn init(rem_addr: SocketAddr, pass: &String) -> DualResult<(), &'static str> {
    let mut error = "";
    let loc_addr = "0.0.0.0:1337";
    let ip = format!("{}", my_internet_ip::get().unwrap());
    let pub_addr = ip.as_str();
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let sock = UdpSocket::bind(&loc_addr.parse().unwrap(), &handle).unwrap();
    let init_sock = SyncUdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1338)).unwrap();

    init_sock.send_to(pub_addr.as_bytes(), &rem_addr).unwrap();
    let (_num, ind_addr) = init_sock.recv_from(&mut [0u8]).unwrap();
    
    let key = handshake(&loc_addr, &ind_addr, &init_sock, pass).unwrap_or_else(|err| {
        error = err;
        Key::from_pw(KeyType::Aes128, pass, loc_addr)
    });

    if error != "" {
        return Err(error);
    }

    let (udp_sink, udp_stream) = sock.framed(UdpVecCodec::new(ind_addr))
    	.split();

    let tun = EncryptedTun::<With<SplitSink<Async>, Vec<u8>, De, Result<Vec<u8>>>, Map<SplitStream<Async>, En>>::new(&key, &handle);

    if tun.is_err() {
        return Err("ERROR: Permission denied. Try running as superuser");
    }

    let (tun_sink, tun_stream) = tun.unwrap().split();

    let sender = tun_stream.forward(udp_sink);
    let receiver = udp_stream.forward(tun_sink);
    core.run(sender.join(receiver))
        .unwrap();
    
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn init(rem_addr: SocketAddr, pass: &String) -> DualResult<(), &'static str> {
    let mut error = "";
    let loc_addr = "127.0.0.1:1337";
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let utun = get_utun().unwrap().bind(loc_addr).unwrap();
    let sock = UdpSocket::from_socket(utun, &handle).unwrap();
    
    let init_sock = SyncUdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1338)).unwrap();

    init_sock.send_to(pub_addr.as_bytes(), &rem_addr).unwrap();
    let (_num, ind_addr) = init_sock.recv_from(&mut [0u8]).unwrap();
    
    let key = handshake(&loc_addr, &ind_addr, &init_sock, pass).unwrap_or_else(|err| {
        error = err;
        Key::from_pw(KeyType::Aes128, pass, loc_addr)
    });

    if error != "" {
        return Err(error);
    }

    let encryptor = En::new(&key);
    let decryptor = De::new(&key);

    let (udp_sink, udp_stream) = UdpFramed.new(sock, UdpVecCodec::new(rem_addr))
    	.split();

    let decrypted_sink = udp_sink.with(decryptor);
    let encrypted_stream = udp_stream.map(encryptor);
    
    core.run(udp_stream.forward(udp_sink))
        .unwrap();
    
    Ok(())
}

fn cmd(cmd: &str, args: &[&str]) {
    let ecode = Command::new("ip")
        .args(args)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
    assert!(ecode.success(), "Failed to execute {}", cmd);
}

pub struct EncryptedTun<T: Sink, U: Stream> {
    sink: T,
    stream: U,
}

impl<T, U> EncryptedTun<T, U>
where T: Sink<SinkItem=Vec<u8>>,
      U: Stream<Item=Vec<u8>>,
      U::Error: std::fmt::Debug,
{
    pub fn new(key: &Key, handle: &Handle) -> DualResult<
        EncryptedTun<
            With<SplitSink<Async>, Vec<u8>, De, Result<Vec<u8>>>,
            Map<SplitStream<Async>, En>
            >,
        &'static str>
        {
        let encryptor = En::new(&key);
        let decryptor = De::new(&key);
        
        let tun = Iface::new("vpn%d", Mode::Tun);

        if tun.is_err() {
                return Err("ERROR: Permission denied. Try running as superuser");
        };
       
        let tun_ok = tun.unwrap();
        cmd("ip", &["addr", "add", "dev", tun_ok.name(), "10.107.1.3/24"]);
        cmd("ip", &["link", "set", "up", "dev", tun_ok.name()]);
        let (sink, stream) = Async::new(tun_ok, handle)
            .unwrap()
            .split();

        let decrypted_sink = sink.with(decryptor);
        let encrypted_stream = stream.map(encryptor);
        
        Ok(EncryptedTun {
            sink: decrypted_sink,
            stream: encrypted_stream,
        })
    }

    pub fn split(self) -> (T, U) {
        (self.sink, self.stream)
    }
}

pub struct UdpVecCodec(SocketAddr);

impl UdpVecCodec {
    pub fn new(addr: SocketAddr) -> Self {
        UdpVecCodec(addr)
    }
}

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
