use std::io::prelude::*;
use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9000")?;

    for stream in listener.incoming() {
        let stream = stream?;
        thread::spawn(|| {
            let _ = handle_incoming(stream);
        });
    }

    Ok(())
}

fn handle_incoming(mut stream: TcpStream) -> io::Result<()> {
    let mut stream_clone = stream.try_clone()?;
    let mut target = TcpStream::connect("127.0.0.1:8080")?;
    let mut target_clone = target.try_clone()?;
    let (tx, rx) = mpsc::channel();
    let tx_1 = tx.clone();
    let tx_2 = tx.clone();

    thread::spawn(move || loop {
        let mut buffer = [0; 512];
        let n = stream_clone.read(&mut buffer).unwrap();
        if n == 0 {
            println!("Request terminated");
            return;
        }
        let event = Event::Request(buffer, n);
        tx_1.send(event).unwrap();
    });

    thread::spawn(move || loop {
        let mut buffer = [0; 512];
        let n = target_clone.read(&mut buffer).unwrap();
        if n == 0 {
            println!("Response terminated");
            return;
        }
        let event = Event::Response(buffer, n);
        tx_2.send(event).unwrap();
    });

    for event in rx {
        match event {
            Event::Request(buffer, n) => {
                let _ = io::stdout().write(&buffer);
                let _ = target.write(&buffer[..n]);
            }
            Event::Response(buffer, n) => {
                let _ = io::stdout().write(&buffer);
                let _ = stream.write(&buffer[..n]);
            }
        }
    }

    Ok(())
}

enum Event {
    Request([u8; 512], usize),
    Response([u8; 512], usize),
}

// https://doc.rust-lang.org/std/net/struct.TcpStream.html
// https://doc.rust-lang.org/std/net/
// https://github.com/sorribas/tcp-spy/blob/master/index.js
