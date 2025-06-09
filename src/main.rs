use serde::{Deserialize, Serialize};
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
    let mut writer = stream.try_clone().unwrap();
    loop {
        let mut buffer: String = String::new();
        match reader.read_line(&mut buffer) {
            Ok(size) if size != 0 => {
                process_request(&mut writer, buffer);
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

fn process_request(writer: &mut TcpStream, buffer: String) {
    match serde_json::from_str::<Request>(&buffer.trim()) {
        Ok(request) => {
            println!("{:?}", request);
            if request.method != "isPrime" {
                send_malformed(writer);
                return;
            }
            let is_prime = check_if_prime(request.number);
            let response = Response {
                method: "isPrime".to_string(),
                prime: is_prime,
            };

            let _ = writer.write(serde_json::to_string(&response).unwrap().as_bytes());
            let _ = writer.write(b"\n");
            let _ = writer.flush();
        }
        Err(x) => {
            println!("ERR: {:?}", x);
            send_malformed(writer);
            return;
        }
    };
}

fn send_malformed(writer: &mut TcpStream) {
    let _ = writer.write_all(String::from("Malformed").as_bytes());
    let _ = writer.write_all(b"\n");
    let _ = writer.flush();
}

fn check_if_prime(num: f64) -> bool {
    if num.fract() != 0.0 {
        return false;
    }
    let num = num.trunc() as i64;
    if num <= 1 {
        return false;
    }

    for i in 2..=((num as f64).sqrt().floor() as i64) {
        if num % i == 0 {
            return false;
        }
    }
    return true;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = check_if_prime(4.0);
        assert_eq!(result, false);
        let result = check_if_prime(13.0);
        assert_eq!(result, true);
        let result = check_if_prime(144.0);
        assert_eq!(result, false);
        let result = check_if_prime(7789.0);
        assert_eq!(result, true);
    }
}
