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
extern crate mac_address;
extern crate serde;
extern crate regex;
extern crate tun_tap;
extern crate bytes;

pub mod rpc;
pub mod tun;
