#![feature(trivial_bounds)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]
#![feature(await_macro, async_await, futures_api)]

#[macro_use]
extern crate tokio;
extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_codec;
extern crate tun_tap;
extern crate miscreant;
extern crate spake2;
extern crate keybob;
extern crate iui;
extern crate mac_utun;
extern crate futures_retry;
extern crate bytes;

pub mod vpn;
pub mod sec;
