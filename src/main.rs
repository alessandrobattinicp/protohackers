use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Deserialize, Debug)]
struct Request {
    method: String,
    number: f64,
}

#[derive(Serialize)]
struct Response {
    method: String,
    prime: bool,
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:5001").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(streamz) => {
                thread::spawn(move || handle_connection(streamz));
            }
            Err(e) => {
                println!("Tcp Error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: TcpStream) {
    println!("Un client si è connesso");
    let connection_stream = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&connection_stream);

    let mut writer = BufWriter::new(&connection_stream);
    loop {
        let mut buffer: String = String::new();
        match reader.read_line(&mut buffer) {
            Ok(size) if size != 0 => {
                match serde_json::from_str::<Request>(&buffer.as_str()) {
                    Ok(request) => {
                        if request.method != "isPrime" {
                            
                            let _ = writer.write("malformed".as_bytes());
                            let _ = writer.write(&[10]);
                        } else {
                            let mut is_prime = false;
                            if request.number > 0.0 {
                                is_prime = primes::is_prime(request.number as u64);
                            }
                            let response = Response {
                                method: "isPrime".to_string(),
                                prime: is_prime,
                            };
                            let _ =
                                writer.write(serde_json::to_string(&response).unwrap().as_bytes());
                            let _ = writer.write(&[10]);
                        }
                    }
                    Err(error) => {
                        println!("Parsing error {:?}", error);
                        let _ = writer.write("malformed".as_bytes());
                        let _ = writer.write(&[10]);
                        break;
                    }
                };
            }
            Ok(_) => {
                println!("Connection closed");

                break;
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }

        match writer.flush() {
            Ok(_) => {}
            Err(e) => {
                println!("errore di flush {:?}", e);
                break;
            }
        }
    }
}