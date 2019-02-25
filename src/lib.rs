#![feature(trivial_bounds)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]

extern crate futures;
extern crate tokio_core;
extern crate tun_tap;
extern crate miscreant;
extern crate spake2;
extern crate keybob;
extern crate iui;
extern crate mac_utun;
extern crate futures_retry;
extern crate my_internet_ip;

pub mod vpn;
pub mod sec;
