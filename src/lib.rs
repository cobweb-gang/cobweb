#![feature(plugin, use_extern_macros)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate tarpc;

extern crate futures;
extern crate tokio_core;
extern crate tokio_codec;
extern crate tokio_io;
extern crate serde;
extern crate tun_tap;
extern crate bytes;
extern crate directories;
extern crate uuid;

pub mod rpc;
pub mod tun;
