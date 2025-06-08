use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
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
            Ok(stream) => {
                thread::spawn(move || handle_connection(stream));
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    let mut writer = stream.try_clone().expect("Failed to clone stream");
    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => {
                println!("Connection closed");
                return;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match serde_json::from_str::<Request>(trimmed) {
                    Ok(request) => {
                        println!("{:?}", request);
                        if request.method != "isPrime" {
                            println!("Invalid method: {:?}", request.method);
                            send_malformed_response(&mut writer);
                            return;
                        }

                        let is_prime = check_if_prime(request.number);
                        let response = Response {
                            method: String::from("isPrime"),
                            prime: is_prime,
                        };

                        let response_json = serde_json::to_string(&response).unwrap();
                        println!("RES: {:?}", response_json);
                        if writer.write_all(response_json.as_bytes()).is_err() {
                            return;
                        }
                        if writer.write_all(b"\n").is_err() {
                            return;
                        }
                        if writer.flush().is_err() {
                            return;
                        }
                    }
                    Err(_) => {
                        println!("Malformed JSON");
                        send_malformed_response(&mut writer);
                        return;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from socket: {}", e);
                return;
            }
        }
    }
}

fn send_malformed_response(writer: &mut TcpStream) {
    let malformed = json!({"error": "malformed"});
    let _ = writer.write_all(malformed.to_string().as_bytes());
    let _ = writer.write_all(b"\n");
    let _ = writer.flush();
}

fn check_if_prime(number: f64) -> bool {
    // Non-integers cannot be prime
    if number.fract() != 0.0 {
        return false;
    }

    let n = number as i64;

    // Numbers less than 2 are not prime
    if n < 2 {
        return false;
    }

    // Check for divisors up to sqrt(n)
    let sqrt_n = (n as f64).sqrt() as i64;
    for i in 2..=sqrt_n {
        if n % i == 0 {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(check_if_prime(4.0), false);
        assert_eq!(check_if_prime(-3.0), false); // Negative numbers are not prime
        assert_eq!(check_if_prime(13.0), true);
        assert_eq!(check_if_prime(144.0), false);
        assert_eq!(check_if_prime(7789.0), true);
        assert_eq!(check_if_prime(2.5), false); // Non-integers are not prime
        assert_eq!(check_if_prime(1.0), false);
        assert_eq!(check_if_prime(2.0), true);
    }
}
