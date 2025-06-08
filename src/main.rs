use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:5001").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Incoming connection");
                thread::spawn(move || handle_connection(stream));
            }
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer: [u8; 512] = [0; 512];
    loop {
        match stream.read(&mut buffer) {
            Ok(size) if size != 0 => {
                if let Err(e) = stream.write_all(&buffer[..size]) {
                    eprintln!("Error writing to socket: {}", e);
                }
            }
            Ok(_) => {
                println!("Connection closed");
                return;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        }
    }
}
