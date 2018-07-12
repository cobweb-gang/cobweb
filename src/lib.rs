pub mod node {
    extern crate mac_address;
    extern crate tokio_ping;
    extern crate futures;
    extern crate tokio;

    use std::fmt;
    use self::mac_address::get_mac_address;
    use self::futures::{Future, Stream};

    pub struct Info {
        mac: mac_address::MacAddress,
        source: NodeType,
    }

    pub enum NodeType {
        Source,
        Link,
    }

    impl fmt::Display for Info {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}\ntype={}", self.mac, self.source)
        }
    }

    impl fmt::Display for NodeType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let print = match *self {
                NodeType::Source => "source",
                NodeType::Link => "link",
            };

            write!(f, "{}", print)
        }
    }

    pub fn get_info(node_type: NodeType) -> Info {
        
        Info {
            mac: get_mac_address().unwrap().unwrap(),
            source: node_type,
        }
    }

    fn _ping() {
        let addr = "1.1.1.1".parse().unwrap();
        let pinger = tokio_ping::Pinger::new();
        let stream = pinger.and_then(move |pinger| Ok(pinger.chain(addr).stream()));
        let future = stream.and_then(|stream| {
            stream.take(3).for_each(|mb_time| {
                match mb_time {
                    Some(time) => println!("time={}", time),
                    None => println!("timeout"),
                }
                Ok(())
            })
        });

        tokio::run(future.map_err(|err| {
            eprintln!("Error: {}", err)
        }))
    }
}
