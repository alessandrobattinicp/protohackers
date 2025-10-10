use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::UdpSocket;
use std::thread;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:5001").unwrap();
    let mut buf: [u8; 1024] = [0; 1024];
    let mut database = HashMap::from([("version".to_string(), "CP client :D 1.0".to_string())]);
    loop {
        match socket.recv_from(&mut buf) {
            Ok((s, sender)) => {
                let mut msg: String = String::from_utf8_lossy(&buf).into(); //mucca munta
                msg.truncate(s);
                println!("Ricevuti {} bytes: {:?}", s, msg);

                if msg.contains("=") {
                    // inster impl
                    let (key, value) = msg.split_once("=").unwrap();
                    if key != "version" {
                        database.insert(key.to_string(), value.to_string());
                    }
                } else {
                    let _ = match database.get(&msg) {
                        Some(value) => {
                            println!("mandando dati: {}={}", msg, value);
                            socket.send_to(format!("{}={}", msg, value).as_bytes(), sender)
                        }
                        None => socket.send_to("Not found".as_bytes(), sender),
                    };
                }
            }

            Err(e) => println!("ERR: {}", e),
        }
    }
}
