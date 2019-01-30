use std::env;
use std::io::prelude::*;
use std::io::{self, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!(include_str!("./usage.txt"));
        return;
    }
    let source_host = format!("127.0.0.1:{}", args[1]);
    let target_port = &args[2];
    let target_host = if target_port.contains(":") {
        format!("{}", target_port)
    } else {
        format!("127.0.0.1:{}", target_port)
    };

    let listener =
        TcpListener::bind(source_host).expect(&format!("Couldn't create server on {}", &args[0]));

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Something wrong with incoming stream: {}", e);
                continue;
            }
        };

        let host = target_host.clone();
        thread::spawn(|| {
            if let Err(e) = handle_incoming(stream, host) {
                eprintln!("{}", e);
            }
        });
    }
}

enum Event {
    FromSource([u8; 512], usize),
    FromTarget([u8; 512], usize),
}

fn handle_incoming(mut source: TcpStream, target_host: String) -> io::Result<()> {
    let mut source2 = source.try_clone()?;
    let mut target = TcpStream::connect(target_host)?;
    let mut target2 = target.try_clone()?;
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();

    // Listen to the source
    thread::spawn(move || loop {
        let mut buffer = [0; 512];
        let n = match source2.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Could not read from source: {}", e);
                continue;
            }
        };
        let _ = tx2.send(Event::FromSource(buffer, n));
        if n == 0 {
            break;
        }
    });

    // Listen to the target
    thread::spawn(move || loop {
        let mut buffer = [0; 512];
        let n = match target2.read(&mut buffer) {
            Ok(n) => n,
            Err(e) => {
                eprintln!("Could not read from target: {}", e);
                continue;
            }
        };
        let _ = tx.send(Event::FromTarget(buffer, n));
        if n == 0 {
            break;
        }
    });

    for event in rx {
        match event {
            Event::FromSource(_, 0) => {
                target.shutdown(Shutdown::Both)?;
                break;
            }
            Event::FromSource(buffer, n) => {
                let _ = io::stdout().write(&buffer[..n]);
                let _ = target.write(&buffer[..n]);
            }
            Event::FromTarget(_, 0) => {
                source.shutdown(Shutdown::Both)?;
                break;
            }
            Event::FromTarget(buffer, n) => {
                let _ = io::stdout().write(&buffer[..n]);
                let _ = source.write(&buffer[..n]);
            }
        }
    }

    Ok(())
}
