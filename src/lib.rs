#![feature(proc_macro_path_invoc)]
#![feature(use_extern_macros)]
#![feature(plugin)]
#![feature(extern_prelude)]
#![plugin(tarpc_plugins)]

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate tarpc;

extern crate futures;
extern crate tokio;
extern crate tokio_core;
extern crate tokio_codec;
extern crate tokio_io;
extern crate rudp;
extern crate serde;
extern crate tun_tap;
extern crate bytes;
extern crate directories;
extern crate uuid;
extern crate hyper;
extern crate httpcodec;
extern crate bytecodec;
extern crate mio;

pub mod rpc;
pub mod tun;
pub mod proxy;
