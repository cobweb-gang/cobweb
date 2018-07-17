extern crate mac_address;
extern crate fibers;
extern crate fibers_rpc;
extern crate mio;
extern crate futures;
extern crate serde;
extern crate bincode;
extern crate bytecodec;
#[cfg(feature = "bincode_codec")]

pub mod node {


    use std::fmt;
    use std::net::IpAddr;
    use ::mac_address::get_mac_address;
    use ::futures::{Future, Stream};
    use ::fibers::{Executor, InPlaceExecutor, Spawn};
    use ::fibers_rpc::{Call, ProcedureId};
    use ::fibers_rpc::client::ClientServiceBuilder;
    use ::bytecodec::bytes::{BytesEncoder, RemainingBytesDecoder};
    use ::bytecodec::bincode_codec::{BincodeEncoder, BincodeDecoder};
    use ::fibers_rpc::server::{HandleCall, Reply, ServerBuilder};
    use ::bincode::{serialize, deserialize};

    // Types

    pub struct Info {
        mac: String,
        node_type: NodeType,
    }

    pub struct InfoRes {
        accepted: bool,
        blacklisted: bool,
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

    impl fmt::Display for InfoRes {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let accept = match self.accepted {
                true => "ACCEPTED",
                false => "REJECTED",
            };
            
            let blacklist = match self.blacklisted {
                true => "Your device has been blacklisted.",
                false => "An error has occurred.",
            };
            
            write!(f, "{}\n{}", accept, blacklist)
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

    impl fmt::Debug for InfoRes {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "InfoRes {{\n accepted: {}\n blacklisted={}\n}}", self.accepted, self.blacklisted)
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
                mac: format!("{}", get_mac_address().unwrap().unwrap()),
                node_type: type_arg,
            }
        }
    }

    // RPCs

    struct InfoRpc;
    impl Call for InfoRpc {
        const ID: ProcedureId = ProcedureId(0);
        const NAME: str = "info";

        type Req = Info;
        type ReqEncoder = BytesEncoder<Info>;
        type ReqDecoder = RemainingBytesDecoder;

        type Res = InfoRes;
        type ResEncoder = BytesEncoder<InfoRes>;
        type ResDecoder = RemainingBytesDecoder;
    }

    struct InfoHandler;
    impl HandleCall<InfoRpc> for InfoHandler {
        fn handle_call(&self, request: <InfoRpc as Call>::Req) -> Reply<InfoRpc> {
            Reply::done(request)
        }
    }

    // Functions

    pub fn start_rpc_server() {
        let mut executor = InPlaceExecutor::new().unwrap();

        let server_addr = "127.0.0.1:1919".parse().unwrap();
        let server = ServerBuilder::new(server_addr)
            .add_call_handler(InfoHandler)
            .finish(executor.handle());
        executor.spawn(server.map_err(|e| panic!("{}", e)));
    }

    pub fn send_info(server_addr: IpAddr) {
        let mut executor = InPlaceExecutor::new().unwrap();
        let service = ClientServiceBuilder::new().finish(executor.handle());

        let request = Info::new(NodeType::Link).unwrap();
        let response = InfoRpc::client(&service.handle()).call(server_addr, request.clone());

        executor.spawn(service.map_err(|e| panic!("{}", e)));
        let result = executor.run_future(response).unwrap();
    }
}
