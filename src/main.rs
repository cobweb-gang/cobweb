extern crate cobweb;
extern crate tokio_core;
extern crate futures;
extern crate tun_tap;
use tokio_core::reactor::Core;
use tokio_core::net::UdpFramed;
use cobweb::vpn::{VpnSocket, UdpVecCodec};
use tun_tap::async::Async;
use futures::prelude::*;
use futures::stream::{SplitSink, SplitStream};

fn main() {
    let loc_addr = "127.0.0.1:1337".parse().unwrap();
    let rem_addr = "127.0.0.1:1338".parse().unwrap();
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    
    let sock = VpnSocket::<SplitSink<Async>, SplitStream<Async>, SplitSink<UdpFramed<UdpVecCodec>>, SplitStream<UdpFramed<UdpVecCodec>>>::connect(&loc_addr, &rem_addr, &handle).unwrap();
    let ((tun_sink, tun_stream), (udp_sink, udp_stream)) = sock.get_all().unwrap();

    let sender = tun_stream.forward(udp_sink);
    let receiver = udp_stream.forward(tun_sink);
    core.run(sender.join(receiver))
        .unwrap();
}
