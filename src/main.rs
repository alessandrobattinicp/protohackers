use serde::{Deserialize, Serialize};
use std::io::{BufReader, BufWriter, Read, Write};
use std::iter::Sum;
use std::net::{TcpListener, TcpStream};
use std::{thread, vec};

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

// FAIL:did not receive at least 5 heartbeats within 10 seconds (only got 0)

fn handle_connection(stream: TcpStream) {
    println!("Un client si è connesso");
    let mut packets: Vec<InsertStruct> = Vec::new();

    let connection_stream = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&connection_stream);
    let mut writer = BufWriter::new(&connection_stream);
    loop {
        let mut op_code: [u8; 1] = [0; 1];
        match reader.read_exact(&mut op_code) {
            Ok(()) => match op_code[0] {
                0x10 => {}
                0x20 => {
                    let plate = parse_plate(&mut reader);
                    println!("plate {:?}", plate);
                }
                0x21 => {}
                0x40 => {
                    let hb = parse_heart_beat(&mut reader);
                    println!("heartbeat {:?}", hb);
                    handle_heartbeat(hb, &connection_stream);
                }
                0x80 => {
                    let camera = parse_i_am_camera(&mut reader);
                    println!("camera {:?}", camera);
                }
                0x81 => {}
                _ => {
                    // eprintln!("Unknown opcode {}", op_code[0]);
                }
            },
            Err(e) => {
                //eprintln!("Read error: {}", e);
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

fn parse_i_am_camera(reader: &mut BufReader<&TcpStream>) -> IAmCamera {
    let mut data: [u8; 6] = [0; 6];
    let _ = reader.read_exact(&mut data);

    IAmCamera {
        road: u16::from_be_bytes([data[0], data[1]]),
        mile: u16::from_be_bytes([data[2], data[3]]),
        limit: u16::from_be_bytes([data[4], data[5]]),
    }
}

fn parse_heart_beat(reader: &mut BufReader<&TcpStream>) -> HeartBeat {
    let mut data: [u8; 4] = [0; 4];
    let _ = reader.read_exact(&mut data);

    HeartBeat {
        interval: u32::from_be_bytes(data),
    }
}

fn parse_plate(reader: &mut BufReader<&TcpStream>) -> Plate {
    let mut plate_length: [u8; 1] = [0; 1];
    let mut timestamp: [u8; 4] = [0; 4];
    let _ = reader.read_exact(&mut plate_length);

    let mut plate_vec: Vec<u8> = vec![0; plate_length[0] as usize];
    let plate = plate_vec.as_mut_slice();

    let _ = reader.read_exact(plate);
    let _ = reader.read_exact(&mut timestamp);

    Plate {
        plate: str::from_utf8(plate).unwrap().to_string(),
        timestamp: u32::from_be_bytes(timestamp),
    }
}

fn handle_heartbeat(hb: HeartBeat, stream: &TcpStream) {
    if hb.interval > 0 {
        let data = [0x41];
        let tmp = stream.try_clone().unwrap();

        thread::spawn(move || {
            let mut writer = BufWriter::new(&tmp);
            loop {
                let _ = writer.write_all(&data);
                let _ = writer.flush();
                std::thread::sleep(std::time::Duration::from_millis(hb.interval as u64 * 100));
            }
        });
    }
}

#[derive(Debug)]
struct IAmCamera {
    road: u16,
    mile: u16,
    limit: u16,
}

#[derive(Debug)]
struct HeartBeat {
    interval: u32,
}

#[derive(Debug)]
struct Plate {
    plate: String,
    timestamp: u32,
}
