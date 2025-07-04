use serde::{Deserialize, Serialize};
use std::io::{BufReader, BufWriter, Read, Write};
use std::iter::Sum;
use std::net::{TcpListener, TcpStream};
use std::thread;

enum OperationType {
    Unknown,
    Insert(InsertStruct),
    Query(QueryStruct),
}

struct InsertStruct {
    timestamp: i32,
    price: i32,
}

struct QueryStruct {
    mintime: i32,
    maxtime: i32,
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
    let mut packets: Vec<InsertStruct> = Vec::new();

    let connection_stream = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&connection_stream);
    let mut writer = BufWriter::new(&connection_stream);
    loop {
        let mut buffer: [u8; 9] = [0; 9];
        match reader.read_exact(&mut buffer) {
            Ok(()) => match buffer.parse() {
                OperationType::Insert(insert) => {
                    packets.push(insert);
                }
                OperationType::Query(query) => {
                    println!(
                        "Query: mintime={}, maxtime={}",
                        query.mintime, query.maxtime
                    );
                    let avg: i32 = average(&packets, query.mintime, query.maxtime);
                    let _ = writer.write_all(&avg.to_be_bytes());
                }
                OperationType::Unknown => {}
            },
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

fn average(packets: &Vec<InsertStruct>, mintime: i32, maxtime: i32) -> i32 {
    if mintime > maxtime {
        return 0;
    }

    let valid_packets: Vec<&InsertStruct> = packets
        .iter()
        .filter(|is| is.timestamp >= mintime && is.timestamp <= maxtime)
        .collect();

    let count = valid_packets.len();
    let total: i64 = valid_packets.iter().map(|is| is.price as i64).sum();

    if count == 0 {
        return 0;
    }

    (total / count as i64) as i32
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
