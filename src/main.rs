extern crate crossbeam;
extern crate toml;
extern crate unix_socket;
extern crate tokio_timer;
extern crate futures;

use std::io::Read;
use std::sync::{Mutex, mpsc};
use unix_socket::UnixDatagram;
use futures::Future;

mod config;

use config::Config;

mod common;

mod work;

use work::CycleWorker;

type MyConfig<'a> = &'a Mutex<Config>;


fn main() {
    let config = Mutex::new(
        Config {
            target: "armv7-unknown-linux-gnueabihf".to_owned()
        }
    );

    let (tx, rx) = mpsc::channel();

    crossbeam::scope(|scope| {
        scope.spawn(|| listen_thread(&config, rx));
        scope.spawn(|| work_thread(&config, tx));
    });
}

// Configuration manager thread
fn listen_thread(config: MyConfig, wakeup_listener: mpsc::Receiver<()>) {
    println!("listen thread");
    let socket = UnixDatagram::bind(common::SOCKET_PATH).unwrap();

    loop {
        let gd = config.lock().unwrap();

        let mut buf = [0; 100];
        let (count, address) = socket.recv_from(&mut buf).unwrap();
        println!("socket {:?} sent {:?}", address, &buf[..count]);
        println!("trying unlock load config");
        drop(gd);

        wakeup_listener.recv().unwrap();
    }
}

// main/working/event loop thread, and check lock to load config when timer arrived
fn work_thread(config: MyConfig, wakeup_notifier: mpsc::Sender<()>) {
    println!("load config thread");
    let do_it = || {
        match config.try_lock() {
            Ok(mut config_gd) => {
                load_config(&mut config_gd);
                wakeup_notifier.send(()).unwrap();
            }
            Err(_) => (),
        }
    };

    let cycle_worker = CycleWorker::new(do_it);
    match cycle_worker.cycle_run().wait() {
        Ok(_) => println!("sleep loop finished"),
        Err(_) => println!("sleep loop failed"),
    };
}

fn load_config(config: &mut Config) {
    let do_open = || -> std::io::Result<String> {
        let mut f = try!(std::fs::File::open("config.toml"));
        let mut buffer = String::new();
        try!(f.read_to_string(&mut buffer));

        Ok(buffer)
    };
    let content = do_open().ok();

    let toml_table = content.as_ref()
        .map(|content| toml::Parser::new(content))
        .and_then(|mut parser| parser.parse());
    let config_data = toml_table.as_ref()
        .and_then(|it| it.get(&"target".to_owned()))
        .and_then(|it| it.as_str());

    if let Some(target_conf) = config_data {
        config.target = target_conf.to_owned();
        println!("load config success: {:#?}", config);
    } else {
        println!("load config failed");
    }
}
