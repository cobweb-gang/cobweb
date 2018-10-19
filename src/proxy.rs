use hyper::{Body, Response, Server};
use hyper::service::service_fn_ok;
use hyper::rt::{self, Future};
use mio::net::UdpSocket;
use rudp::Endpoint;
use std::io::Write;
use futures::Stream;
use bytes::Bytes;
use bytes::buf::FromBuf;
use bytecodec::Encode;
use bytecodec::bytes::BytesEncoder;
use bytecodec::io::IoEncodeExt;
use httpcodec::{BodyEncoder, HttpVersion, Method, Request, RequestEncoder, RequestTarget};

static PHRASE: &'static [u8] = b"Hello World!";

pub fn start_proxy() {

    let proxy = || {
        service_fn_ok(|req: hyper::Request<hyper::Body>| {
            println!("{:?}", req);
            let udp_addr = "127.0.0.1:1338".parse().unwrap();
            let tun_addr = "127.0.0.1:6969".parse().unwrap();
            let sock = UdpSocket::bind(&udp_addr).unwrap();
            sock.connect(tun_addr).unwrap();
            let mut endpt = Endpoint::new(sock);
            let method = req.method().as_str();
            let uri = req.uri().to_string();
            let request: Request<&[u8]> = Request::new(
                Method::new(&method).unwrap(),
                RequestTarget::new(&uri).unwrap(),
                HttpVersion::V1_1,
                b"IGNORE"
            );
            println!("{:?}", request.header());
            let mut encoder = RequestEncoder::new(BodyEncoder::new(BytesEncoder::new()));
            encoder.start_encoding(request).unwrap();

            let mut buf = Vec::new();
            encoder.encode_all(&mut buf).unwrap();
            let bytes = Bytes::from_buf(&buf);

            endpt.write(bytes.as_ref()).unwrap();
          
            // COLLECT IT INTO A VEC AND SEND VIA A .NEXT()
            req.body().collect().wait().unwrap().into_iter().for_each(move |chunk| {
                endpt.write(chunk.into_bytes().as_ref()).unwrap();
            });
            Response::new(Body::from(PHRASE))
        })
    };

    let addr = "127.0.0.1:1337".parse().unwrap();
    let server = Server::bind(&addr)
        .serve(proxy);

    rt::run(server.map_err(|e| eprintln!("server error: {}", e)));
}
