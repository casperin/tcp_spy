use std::io::prelude::*;
use std::io::{self, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9000")?;

    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(|| {
            if let Err(e) = handle_incoming(stream) {
                println!("{}", e);
            }
        });
    }

    Ok(())
}

enum Event {
    Request([u8; 512], usize),
    Response([u8; 512], usize),
}

fn handle_incoming(mut source: TcpStream) -> io::Result<()> {
    let mut source2 = source.try_clone()?;
    let mut target = TcpStream::connect("127.0.0.1:8080")?;
    let mut target2 = target.try_clone()?;
    let (tx, rx) = mpsc::channel();
    let tx2 = tx.clone();

    thread::spawn(move || loop {
        let mut buffer = [0; 512];
        let n = source2.read(&mut buffer).unwrap();
        let _ = tx2.send(Event::Request(buffer, n));
        if n == 0 {
            break;
        }
    });

    thread::spawn(move || loop {
        let mut buffer = [0; 512];
        let n = target2.read(&mut buffer).unwrap();
        let _ = tx.send(Event::Response(buffer, n));
        if n == 0 {
            break;
        }
    });

    for event in rx {
        match event {
            Event::Request(_, 0) => {
                target.shutdown(Shutdown::Both)?;
                break;
            }
            Event::Response(_, 0) => {
                source.shutdown(Shutdown::Both)?;
                break;
            }
            Event::Request(buffer, n) => {
                let _ = io::stdout().write(&buffer);
                let _ = target.write(&buffer[..n]);
            }
            Event::Response(buffer, n) => {
                let _ = io::stdout().write(&buffer);
                let _ = source.write(&buffer[..n]);
            }
        }
    }

    Ok(())
}
