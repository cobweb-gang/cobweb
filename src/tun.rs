pub mod tun {
    use std::io;
    use futures::Stream;
    use tokio_core::net::{TcpListener};
    use tokio_core::reactor::Core;
    use tokio_codec::{Encoder, Decoder};
    use tokio_io::AsyncRead;
    use tun_tap::{Iface, Mode};
    use tun_tap::async::Async;
    use bytes::BytesMut;

    struct VecCodec;

    impl Decoder for VecCodec {
        type Item = Vec<u8>;
        type Error = io::Error;

        fn decode (&mut self, buf: &mut BytesMut) -> io::Result<Option<Self::Item>> {
            Ok(Some(buf.iter().cloned().collect()))
        }
    }

    impl Encoder for VecCodec {
        type Item = Vec<u8>;
        type Error = io::Error;

        fn encode (&mut self, item: Self::Item, dst: &mut BytesMut) -> io::Result<()> {
            dst.extend_from_slice(item.as_slice());

            Ok(())
        }
    }

    pub fn start_tun() {
        let mut core = Core::new().unwrap();
        let handle = core.handle();
        let loc_address = "127.0.0.1:6969".parse().unwrap();
        let listener = TcpListener::bind(&loc_address, &handle)
            .unwrap();
        let server = listener.incoming().for_each(|(sock, _)| {
            let (sender, receiver) = sock.framed(VecCodec)
                .split();
            let tun = Iface::new("vpn%d", Mode::Tun)
                .unwrap();
            let (sink, stream) = Async::new(tun, &handle)
                .unwrap()
                .split();
            let reader = stream.forward(sender);
            let writer = receiver.forward(sink);

            Ok(())
        });

        core.run(server).unwrap();
    }
}
