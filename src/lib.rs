#![feature(plugin, use_extern_macros)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate tarpc;

extern crate futures;
extern crate tokio_core;
extern crate mac_address;
extern crate serde;
extern crate regex;

pub mod node {

    use std::fmt;
    use std::collections::HashSet;
    use ::regex::Regex;
    use ::mac_address::get_mac_address;
    use ::futures::Future;
    use ::tarpc::future::{client, server};
    use ::tarpc::future::client::ClientExt;
    use ::tarpc::util::{FirstSocketAddr, Never};
    use ::tokio_core::reactor;

    // Types

    #[derive(Serialize, Deserialize, Clone)]
    pub struct Info {
        mac: String,
        node_type: NodeType,
        status: Status,
    }
    
    #[derive(Serialize, Deserialize, Clone, Copy)]
    pub enum NodeType {
        Source,
        Link,
    }

    #[derive(Serialize, Deserialize, Clone)]
    pub enum Status {
        Req,
        Acc,
        Rej,
    }

    // Implementations

    impl fmt::Display for Info {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}\ntype={}\nstats={}", self.mac, self.node_type, self.status)
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

    impl fmt::Display for Status {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let print = match *self {
                Status::Req => "REQUESTING",
                Status::Acc => "ACCEPTING",
                Status::Rej => "REJECTING",
            };

            write!(f, "{}", print)
        }
    }

    impl fmt::Debug for Info {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Info {{\n mac: {}\n node_type={}\n status={}\n}}", self.mac, self.node_type, self.status)
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

    impl fmt::Debug for Status {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let print = match *self {
                Status::Req => "Status::Req",
                Status::Acc => "Status::Acc",
                Status::Rej => "Status::Rej",
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
                mac: format!("{}", get_mac_address().unwrap().unwrap()),
                node_type: type_arg,
                status: Status::Req,
            }
        }

        pub fn eval(info: Info, type_arg: NodeType, blacklist: &HashSet<String>) -> Info {
            let re = Regex::new(r"^([[:xdigit:]]{2}[:.-]?){5}[[:xdigit:]]{2}$").unwrap();
            let mut result: Info;

            if !re.is_match(&info.mac.as_str()) {
                println!("Not a MAC address, brah");
                result = Info {
                    mac: format!("{}", get_mac_address().unwrap().unwrap()),
                    node_type: type_arg,
                    status: Status::Rej,
                };
            }
            
            if blacklist.contains(&info.mac) {
                result = Info {
                    mac: format!("{}", get_mac_address().unwrap().unwrap()),
                    node_type: type_arg,
                    status: Status::Rej,
                };
            } else {
                result = Info {
                    mac: format!("{}", get_mac_address().unwrap().unwrap()),
                    node_type: type_arg,
                    status: Status::Acc,
                };
            }

            result
        }
    }

    // RPC Stuff

    service! {
        rpc info(info: Info) -> Info;
    }

	#[derive(Clone)]
	struct InfoServer;

	impl FutureService for InfoServer {
		type InfoFut = Result<Info, Never>;

		fn info(&self, info: Info) -> Self::InfoFut {
            println!("Bruh! The info function was called");

			Ok(Info::eval(info, NodeType::Link, &HashSet::new()))
		}
	}

	pub fn send_info() {
		let mut reactor = reactor::Core::new().unwrap();
		let (handle, server) = InfoServer.listen("localhost:10000".first_socket_addr(),
		&reactor.handle(),
		server::Options::default())
			.unwrap();
		reactor.handle().spawn(server);
		let options = client::Options::default().handle(reactor.handle());
		reactor.run(FutureClient::connect(handle.addr(), options)
					.map_err(::tarpc::Error::from)
					.and_then(|client| client.info(Info::new(NodeType::Link)))
					.map(|resp| println!("{}", resp)))
			.unwrap();
	}

    // Tests

    #[test]
    fn info_eval() {
        let info = Info::new(NodeType::Link);
        let mut list = HashSet::new();
        let info_res = Info::eval(info.clone(), NodeType::Link, &list);
        list.insert(String::from("00:1D:72:8E:C9:AE"));
        let info_res_blacklisted = Info::eval(info, NodeType::Link, &list);

        assert_eq!(
            format!("{}", info_res.status),
            "ACCEPTING"
            );
        
        assert_eq!(
            format!("{}", info_res_blacklisted.status),
            "REJECTING"
            );
    }
}
