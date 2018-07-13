#![deny(warnings)]

/* extern crate tokio;
extern crate cobweb;

use std::sync::{Arc, Mutex};
use std::env;
use std::net::{Shutdown, SocketAddr};
use std::io::{self, Read, Write};
use cobweb::node::get_info;

use tokio::io::{copy, shutdown};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
*/

extern crate cobweb;
use cobweb::node::Info;
use cobweb::node::NodeType;

fn main() {
    println!("{}", Info::new(NodeType::Source));
}

/* fn main() {
    let listen_addr = env::args().nth(1).unwrap_or("127.0.0.1:8081".to_string());
    let listen_addr = listen_addr.parse::<SocketAddr>().unwrap();

    let server_addr = env::args().nth(2).unwrap_or("127.0.0.1:8080".to_string());
    let server_addr = server_addr.parse::<SocketAddr>().unwrap();

    let socket = TcpListener::bind(&listen_addr).unwrap();
    println!("Listening on: {}", listen_addr);
    println!("Proxying to: {}", server_addr);

    let done = socket.incoming()
        .map_err(|e| println!("error accepting socket; error = {:?}", e))
        .for_each(move |client| {
            let server = TcpStream::connect(&server_addr);
            let amounts = server.and_then(move |server| {
                let client_reader = MyTcpStream(Arc::new(Mutex::new(client)));
                let client_writer = client_reader.clone();
                let server_reader = MyTcpStream(Arc::new(Mutex::new(server)));
                let server_writer = server_reader.clone();

                let client_to_server = copy(client_reader, server_writer)
                    .and_then(|(n, _, server_writer)| {
                        shutdown(server_writer).map(move |_| n)
                    });

                let server_to_client = copy(server_reader, client_writer)
                    .and_then(|(n, _, client_writer)| {
                        shutdown(client_writer).map(move |_| n)
                    });

                client_to_server.join(server_to_client)
            });

            let msg = amounts.map(move |(from_client, from_server)| {
                println!("client wrote {} bytes and received {} bytes",
                         from_client, from_server);
            }).map_err(|e| {
                println!("error: {}", e);
            });

            tokio::spawn(msg);

            Ok(())
        });

    tokio::run(done);
}

#[derive(Clone)]
struct MyTcpStream(Arc<Mutex<TcpStream>>);

impl Read for MyTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.lock().unwrap().read(buf)
    }
}

impl Write for MyTcpStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl AsyncRead for MyTcpStream {}

impl AsyncWrite for MyTcpStream {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        try!(self.0.lock().unwrap().shutdown(Shutdown::Write));
        Ok(().into())
    }
} */
