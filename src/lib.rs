pub mod node {
    extern crate mac_address;
    extern crate tokio_ping;
    extern crate futures;
    extern crate tokio;

    use std::fmt;
    use self::mac_address::get_mac_address;
    use self::futures::{Future, Stream};

    // Types

    pub struct Info {
        mac: mac_address::MacAddress,
        node_type: NodeType,
    }

    pub enum NodeType {
        Source,
        Link,
    }

    // Implementations

    impl fmt::Display for Info {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}\ntype={}", self.mac, self.node_type)
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

    impl fmt::Debug for Info {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Info {{\n mac: {}\n node_type={}\n}}", self.mac, self.node_type)
        }
    }

    impl fmt::Debug for NodeType {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let print = match *self {
                NodeType::Source => "NodeType::Source",
                NodeType::Link => "NodeType::Link",
            };

            write!(f, "{}", print)
        }
    }

    impl PartialEq for Info {
        fn eq(&self, other: &Info) -> bool {
            self == other
        }
    }

    impl Info {
        pub fn new(type_arg: NodeType) -> Info {
            Info {
                mac: get_mac_address().unwrap().unwrap(),
                node_type: type_arg,
            }
        }
    }

    // Functions

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
