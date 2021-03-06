use crate::sec::en::{En, De};
use crate::sec::client::handshake;
use std::io::Result;
use keybob::Key;
use tun_tap::{Iface, Mode};
use tun_tap::r#async::Async;
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
    
    let key = handshake(&loc_addr, &rem_addr, &sock, pass);
    rem_addr.set_port(u16::from_be_bytes(client_num));
   
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
    #[cfg(target_os = "linux")]
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

    #[cfg(target_os = "macos")]
    pub fn new(key: &Key, handle: &Handle) -> DualResult<
        EncryptedTun<
            With<SplitSink<tun_tap_mac::r#async::Async>, Vec<u8>, De, Result<Vec<u8>>>,
            Map<SplitStream<tun_tap_mac::r#async::Async>, En>
            >,
        &'static str>
        {
        let encryptor = En::new(&key);
        let decryptor = De::new(&key);
        
        let tun = tun_tap_mac::Iface::new("vpn%d", tun_tap_mac::Mode::Tun);

        if tun.is_err() {
                return Err("ERROR: Permission denied. Try running as superuser");
        };
       
        let tun_ok = tun.unwrap();
        cmd("ip", &["addr", "add", "dev", tun_ok.name(), "10.107.1.3/24"]);
        cmd("ip", &["link", "set", "up", "dev", tun_ok.name()]);
        let (sink, stream) = tun_tap_mac::r#async::Async::new(tun_ok, handle)
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
