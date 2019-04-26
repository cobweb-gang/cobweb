use crate::sec::en::{En, De};
use crate::sec::client::handshake;
use std::io::Result;
use keybob::Key;
use tun_tap::{Iface, Mode};
use tun_tap::r#async::Async;
use mac_utun::get_utun;
use std::process::Command;
use std::net::SocketAddr;
use tokio_core::reactor::{Core, Handle};
use tokio_core::net::TcpStream;
use tokio_codec::{Decoder, Encoder};
use tokio_io::io::read;
use tokio_io::AsyncRead;
use futures::prelude::*;
use futures::stream::{SplitSink, SplitStream};
use futures::sink::With;
use futures::stream::Map;
use bytes::BytesMut;
use std::result::Result as DualResult;
use std::net::Shutdown;

#[cfg(target_os = "linux")]
pub fn init(mut rem_addr: SocketAddr, pass: &String) -> DualResult<(), &'static str> {
    let loc_addr = "0.0.0.0:1337";
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let sock = TcpStream::connect(&rem_addr, &handle).wait();
   
    let sock = match sock {
        Ok(sock) => sock,
        Err(_e) => return Err("ERROR: Connection refused. Do you have the correct IP address?"),
    };

    let mut client_num = [0u8; 2];
    read(&sock, &mut client_num).wait().unwrap();
    rem_addr.set_port(u16::from_be_bytes(client_num));
    
    let key = handshake(&loc_addr, &rem_addr, &sock, pass);
   
    let key = match key {
        Ok(key) => key,
        Err(e) => return Err(e),
    };

    sock.shutdown(Shutdown::Both).unwrap();
    let sock2 = TcpStream::connect(&rem_addr, &handle).wait().unwrap();
    
    let (tcp_sink, tcp_stream) = sock2.framed(TcpVecCodec)
    	.split();

    let tun = EncryptedTun::<With<SplitSink<Async>, Vec<u8>, De, Result<Vec<u8>>>, Map<SplitStream<Async>, En>>::new(&key, &handle);
   
    let tun = match tun {
        Ok(tun) => tun,
        Err(e) => return Err(e),
    };

    let (tun_sink, tun_stream) = tun.split();

    let sender = tun_stream.forward(tcp_sink);
    let receiver = tcp_stream.forward(tun_sink);
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

struct TcpVecCodec;

impl Decoder for TcpVecCodec {
    type Item = Vec<u8>;
    type Error = std::io::Error;

    fn decode (&mut self, buf: &mut BytesMut) -> std::io::Result<Option<Self::Item>> {
        Ok(Some(buf.iter().cloned().collect()))
    }
}

impl Encoder for TcpVecCodec {
    type Item = Vec<u8>;
    type Error = std::io::Error;

    fn encode (&mut self, item: Self::Item, dst: &mut BytesMut) -> std::io::Result<()> {
        dst.extend_from_slice(item.as_slice());

        Ok(())
    }
}
