#![feature(trivial_bounds)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]

extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_codec;
extern crate tun_tap;
extern crate tun_tap_mac;
extern crate miscreant;
extern crate spake2;
extern crate keybob;
extern crate iui;
extern crate futures_retry;
extern crate bytes;

pub mod vpn;
pub mod sec;
