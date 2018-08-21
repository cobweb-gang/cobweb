use std::fmt;
use std::fs;
use ::futures::Future;
use ::tarpc::future::{client, server};
use ::tarpc::future::client::ClientExt;
use ::tarpc::util::{FirstSocketAddr, Never};
use ::tokio_core::reactor;
use ::uuid::Uuid;
use ::directories::ProjectDirs;

// Types

#[derive(Serialize, Deserialize, Clone)]
pub struct Info {
    uuid: String,
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
        write!(f, "{}\ntype={}\nstats={}", self.uuid, self.node_type, self.status)
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
        write!(f, "Info {{\n uuid: {}\n node_type={}\n status={}\n}}", self.uuid, self.node_type, self.status)
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
            uuid: Info::uuid(),
            node_type: type_arg,
            status: Status::Req,
        }
    }

    pub fn uuid() -> String {
        let data_dir = ProjectDirs::from("org", "Cobweb Gang", "cobweb").unwrap();
        let mut dir = data_dir.data_dir().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        dir.push("uu.id");
        println!("{:?}", dir);
        let result: String;

        result = fs::read_to_string(&dir).unwrap_or_else(|_err| {
            let uuid = Uuid::new_v4();

            fs::write(dir, uuid.simple().to_string()).unwrap();
            uuid.simple().to_string()
        });

        result
    }

    pub fn eval(_info: Info, type_arg: NodeType) -> Info {
        Info {
            uuid: Info::uuid(),
            node_type: type_arg,
            status: Status::Acc,
        }
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

        Ok(Info::eval(info, NodeType::Link))
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
fn save_uuid() {
    let data_dir = ProjectDirs::from("org", "Cobweb Gang", "cobweb").unwrap();
    let uuid_dir = format!("{:?}/uu.id", data_dir.data_dir());
    let id = Info::uuid();
    // let read_id = fs::read_to_string(data_dir.data_dir()).unwrap();

    /* assert_eq!(
       id,
       read_id
       ); */
}
