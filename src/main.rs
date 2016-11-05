extern crate crossbeam;
extern crate toml;
extern crate unix_socket;

use std::io::Read;
use std::sync::Mutex;
use unix_socket::UnixDatagram;

mod config;
use config::Config;
mod common;

fn main() {
    let config = Mutex::new(Config { target: "armv7-unknown-linux-gnueabihf".to_owned() });

    crossbeam::scope(|scope| {
        scope.spawn(|| listen_thread(&config));
        scope.spawn(|| load_config_thread(&config));
    });
}

// Configuration manager thread
fn listen_thread(config: &Mutex<Config>) {
    println!("listen thread");
    let socket = UnixDatagram::bind(common::SOCKET_PATH).unwrap();

    loop {
        let gd = config.lock().unwrap();

        let mut buf = [0; 100];
        let (count, address) = socket.recv_from(&mut buf).unwrap();
        println!("socket {:?} sent {:?}", address, &buf[..count]);
        println!("trying unlock load config");
        drop(gd);
    }
}

// main/working/event loop thread
fn load_config_thread(config: &Mutex<Config>) {
    println!("load config thread");
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let mut config_gd = config.lock().unwrap();
        load_config(&mut config_gd);
    }
}

fn load_config(config: &mut Config) {
    let do_open = || -> std::io::Result<Vec<u8>> {
        let mut f = try!(std::fs::File::open("config.toml"));
        let mut buffer = Vec::<u8>::new();
        try!(f.read_to_end(&mut buffer));

        Ok(buffer)
    };
    let content = do_open().ok().and_then(|it| String::from_utf8(it).ok());

    let mut parser = toml::Parser::new(content.as_ref().map(|it| it.as_str()).unwrap_or(""));
    let toml_table = parser.parse();
    let config_data = toml_table.as_ref()
        .and_then(|it| it.get(&"target".to_owned()))
        .and_then(|it| it.as_str());

    if let Some(target_config) = config_data {
        config.target = target_config.to_owned();
        println!("load config success: {:#?}", target_config);
    } else {
        println!("load config failed");
    }
}
