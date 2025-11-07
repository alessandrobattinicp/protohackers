use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read, Write};
use std::iter::Sum;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::{thread, vec};

#[derive(Debug, PartialEq, Clone, Copy)]
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

struct CarDetected {
    camera: IAmCamera,
    timestamp: u32,
}

// impl Ord for CarDetected {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         return self.timestamp.cmp(&other.timestamp);
//     }

// }

// assert_eq!(5.cmp(&10), Ordering::Less);
// assert_eq!(10.cmp(&5), Ordering::Greater);
// assert_eq!(5.cmp(&5), Ordering::Equal);

struct Ticket {
    plate: String,
    road: u16,
    mile1: u16,
    timestamp1: u32,
    mile2: u16,
    timestamp2: u32,
    speed: u16,
}

struct Detection {
    map: HashMap<String, Vec<CarDetected>>,
}

impl Detection {
    fn insert_with_detection(&mut self, plate: String, car: CarDetected) -> Option<Ticket> {
        match self.map.get_mut(plate.as_str()) {
            None => {
                self.map.insert(plate, vec![car]);
                return None;
            }
            Some(vec) => {
                vec.push(car);

                vec.sort();
            }
        }
    }
}

type Detections = Arc<Mutex<Detection>>;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:5001").unwrap();

    let detections: Detections = Arc::new(Mutex::new(Detection {
        map: HashMap::new(),
    }));

    for stream in listener.incoming() {
        let detections = Arc::clone(&detections);
        match stream {
            Ok(streamz) => {
                thread::spawn(move || handle_connection(streamz, detections));
            }
            Err(e) => {
                println!("Tcp Error: {}", e);
            }
        }
    }
}

// FAIL:did not receive at least 5 heartbeats within 10 seconds (only got 0)

fn handle_connection(stream: TcpStream, detections: Detections) {
    println!("Un client si è connesso");

    let connection_stream = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&connection_stream);
    let mut writer = BufWriter::new(&connection_stream);
    let mut camera = Option::<IAmCamera>::None;

    loop {
        let mut op_code: [u8; 1] = [0; 1];
        match reader.read_exact(&mut op_code) {
            Ok(()) => match op_code[0] {
                0x20 => match camera {
                    None => {
                        println!("error (isNotCamera)");
                        let _ = writer.write_all(&[10, 3, 62, 61, 64]);
                        let _ = writer.flush();
                    }
                    Some(camera) => {
                        let plate = parse_plate(&mut reader);
                        println!("plate {:?}", plate);

                        let mut detections = detections.lock().unwrap();

                        detections.insert_with_detection(
                            plate.plate,
                            CarDetected {
                                camera,
                                timestamp: plate.timestamp,
                            },
                        );
                    }
                },
                0x21 => {}
                0x40 => {
                    let hb = parse_heart_beat(&mut reader);
                    println!("heartbeat {:?}", hb);
                    handle_heartbeat(hb, &connection_stream);
                }
                0x80 => {
                    //TODO: check if switching type
                    camera = Some(parse_i_am_camera(&mut reader));
                    println!("camera {:?}", camera);
                }
                0x81 => {} //TODO: check if switching type
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

// [Fri Nov  7 15:33:05 2025 UTC] [1car.test] FAIL:didn't receive speeding ticket within 10 seconds

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
