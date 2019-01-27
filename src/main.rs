use std::io::prelude::*;
use std::io::{self, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

fn main() -> io::Result<()> {
    let (tx, rx) = mpsc::channel();
    let listener = TcpListener::bind("127.0.0.1:9000")?;

    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut buffer = [0; 512];
            stream.read(&mut buffer).unwrap();
            let event = Event::Server(buffer);
            tx.send(event).unwrap();
        }
    });

    for event in rx {
        match event {
            Event::Server(buffer) => {
                // Handle server requests
                io::stdout().write(&buffer);
                // ... send to client
            }
            _ => {
                // Handle client response
            }
        }
    }

    Ok(())
}

enum Event {
    Server([u8; 512]),
    Client([u8; 512]),
}

/*
fn handle(mut stream: TcpStream) {
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}
*/

// https://doc.rust-lang.org/std/net/struct.TcpStream.html
// https://doc.rust-lang.org/std/net/
// https://github.com/sorribas/tcp-spy/blob/master/index.js
//
//
// let response = "HTTP/1.1 200 OK\r\n\r\n";
// stream.write(response.as_bytes()).unwrap();
// stream.flush().unwrap();
