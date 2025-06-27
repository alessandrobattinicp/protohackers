use serde::{Deserialize, Serialize};
use std::io::{BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

enum OperationType {
    Unknown,
    Insert(InsertStruct),
    Query(QueryStruct),
}

struct InsertStruct {
    timestamp: i32,
    price: i32
}

struct QueryStruct {
    mintime: i32,
    maxtime: i32
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
        let mut buffer:[u8;9] = [0;9];
        match reader.read(&mut buffer) {
            Ok(size) if size != 0 => {
                println!("Ricevuto: {:02X}, {:?}", buffer[0], buffer);
                match buffer.parse() {
                    OperationType::Insert(insert) => {
                        println!("Insert: timestamp={}, price={}", insert.timestamp, insert.price);
                        // Here you would handle the insert operation, e.g., store it in a database
                        //writer.write_all(b"Insert received\n").unwrap();
                    }
                    OperationType::Query(query) => {
                        println!("Query: mintime={}, maxtime={}", query.mintime, query.maxtime);
                        // Here you would handle the query operation, e.g., retrieve data from a database
                        //writer.write_all(b"Query received\n").unwrap();
                    }
                    OperationType::Unknown => {
                        eprintln!("Unknown operation");
                        //writer.write_all(b"Unknown operation\n").unwrap();
                    }
                }
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

        /*match writer.flush() {
            Ok(_) => {}
            Err(e) => {
                println!("errore di flush {:?}", e);
                break;
            }
        }*/
    }
}

trait ParseOperation {
    fn parse(&self) -> OperationType;
}

impl ParseOperation for [u8; 9] {
    fn parse(&self) -> OperationType {
        match self[0] as char {
            'I' => {
                let timestamp = i32::from_be_bytes([self[1], self[2], self[3], self[4]]);
                let price = i32::from_be_bytes([self[5], self[6], self[7], self[8]]);
                OperationType::Insert(InsertStruct { timestamp, price })
            }
            'Q' => {
                let mintime = i32::from_be_bytes([self[1], self[2], self[3], self[4]]);
                let maxtime = i32::from_be_bytes([self[5], self[6], self[7], self[8]]);
                OperationType::Query(QueryStruct { mintime, maxtime })
            }
            _ => OperationType::Unknown,
        }
    }
}
