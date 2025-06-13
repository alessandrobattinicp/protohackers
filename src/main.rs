use serde::{Deserialize, Serialize};
use std::io::{BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Deserialize, Debug)]
struct Request {
    method: String,
    number: f32,
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
                println!("Error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer: [u8; 512] = [0; 512];
    loop {
        let reader = BufReader::new(stream.try_clone().unwrap());
        match reader.read_line(&mut buffer) {
            Ok(size) if size != 0 => {
                println!("Buffer : {:?}", buffer);
                match serde_json::from_slice::<Request>(&buffer[..size]) {
                    Ok(request) => {
                        println!("{:?}", request);
                        if request.method != "isPrime" {
                            println!("richiesta malformata, {:?}", request);
                            let _ = stream.write("malformed".as_bytes());
                            let _ = stream.write(&[10]);
                        } else {
                            let is_prime = check_if_prime(request.number).unwrap();
                            let response = Response {
                                method: "isPrime".to_string(),
                                prime: is_prime,
                            };
                            let _ =
                                stream.write(serde_json::to_string(&response).unwrap().as_bytes());
                            let _ = stream.write(&[10]);
                        }
                    }
                    Err(error) => {
                        println!("Errore {:?}", error);
                        let _ = stream.write("malformed".as_bytes());
                        let _ = stream.write(&[10]);
                        break;
                    }
                };
            }
            Ok(_) => {
                println!("Connection closed");
                break;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}

fn check_if_prime(number: f32) -> Option<bool> {
    if number.fract() != 0.0 {
        return Some(false);
    }

    let num = number as i32;

    if num < 2 {
        return Some(false);
    }
    let has_divisor = (2..=num / 2).any(|x| num % x == 0);
    Some(!has_divisor)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = check_if_prime(13.4).unwrap();
        assert_eq!(result, false);
        let result = check_if_prime(4.0).unwrap();
        assert_eq!(result, false);
        let result = check_if_prime(-4.0).unwrap();
        assert_eq!(result, false);
        let result = check_if_prime(13.0).unwrap();
        assert_eq!(result, true);
        let result = check_if_prime(144.0).unwrap();
        assert_eq!(result, false);
        let result = check_if_prime(7789.0).unwrap();
        assert_eq!(result, true);
    }
}
