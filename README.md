# rust_mc_serv

Simple Minecraft server (java edition) written in pure rust. 
 
### Supported versions

- 1.21.5 (PVN 770)

## How to run

First of all, you need to get a binary, that can be done using these methods:

### Download from release

At the moment, there are no releases as the project is in developement

If you want to build the latest binary on your own, please refer to the following method.

### Build it yourself

To build the project by yourself, you should to:

1. Download and install [Rust](https://www.rust-lang.org/)
2. Download source code of the project (from zip or `git clone`)
3. Open terminal in the project folder and run the following commands:

To run:
```bash
cargo run
```

To build (the built binary will be in `target/release`):
```bash
cargo build -r
```

## Use as a library

You can use the project as a library in your servers

Example of adding in `Cargo.toml`:

```toml
rust_mc_serv = { git = "https://github.com/MeexReay/rust_mc_serv.git" }
```

Example of running a server:

```rust
let config = Arc::new(Config::default());
let mut server = ServerContext::new(config);

// Adding default Play mode handling
server.add_packet_handler(Box::new(PlayHandler)); 
server.add_listener(Box::new(PlayListener));

server.add_listener(Box::new(ExampleListener)); // Adding listener example
server.add_packet_handler(Box::new(ExamplePacketHandler)); // Adding packet handler example

start_server(Arc::new(server));
```

## Configuration

By default, the config will be created in `config.toml` in the working directory. To change this path, specify it as the first argument to the server, example: `./rust_mc_serv /path/to/config.toml`

## Contributing

If you would like to contribute to the project, feel free to fork the repository and submit a pull request.

## License
This project is licensed under the WTFPL License
