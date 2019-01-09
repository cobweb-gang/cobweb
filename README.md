# Cobweb

Cobweb is a VPN software suite based on a fast and simple custom VPN protocol, designed for building a decentralized federation of self-hosted servers.

Currently Linux and MacOS are supported. Windows support is a long-term goal but not currently planned.

## Build & Install

First, install the Rust toolchain if you haven't already. Run the command below and follow the directions presented to you

`curl https://sh.rustup.rs -sSf | sh`

Once you've installed Rust, clone this repository

`git clone https://github.com/cobweb-gang/cobweb`

Move into the repository and compile the code

```
cd cobweb
cargo build --release
```

Once you've done this, you can run Cobweb as such:

`target/release/cobweb`

Alternatively, you can move the binary into your $PATH and run it without a directory prefix:

```
mv target/release/cobweb /usr/local/bin/cobweb
cobweb
```

Currently, the software is unusable as there is no server currently being hosted. In the beta release, a build option will be added in order to discern between building the client or server version of Cobweb, so the build and installation process is subject to change.

Pre-compiled binaries for Linux and MacOS will be provided with the first beta release.
