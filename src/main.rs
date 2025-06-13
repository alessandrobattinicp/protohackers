use std::io::BufReader;

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:5001").await?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_connection(stream));
            }
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer: [u8; 2048] = [0; 2048];
    loop {
        stream.tr
        let reader = BufReader::new(stream.try_clone().unwrap());
        match reader.read_line(&mut buffer).await {
            Ok(size) if size != 0 => {
                println!("buffer: {:?}", std::str::from_utf8(&buffer[..size]));
                match serde_json::from_slice::<Request>(&buffer[..size]) {
                    Ok(request) => {
                        println!("{:?}",request);
                        if request.method != "isPrime" {
                            println!("richiesta malformata, {:?}", request);
                            let _ = stream.write("malformed".as_bytes()).await;
                            let _ = stream.write(&[10]).await;
                            break;
                        }
                        let is_prime = check_if_prime(request.number).unwrap();
                        let response = Response {method:"isPrime".to_string(), prime: is_prime};
                        let _ = stream.write(serde_json::to_string(&response).unwrap().as_bytes()).await;
                        let _ = stream.write(&[10]).await;
                    }
                    Err(error) => {
                        println!("Error{:?}",error);
                        let _ = stream.write("malformed".as_bytes());
                        let _ = stream.write(&[10]).await;
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

fn u32_vec_to_string(vec: &[u32]) -> String {
    vec.iter()
        .filter_map(|&code| std::char::from_u32(code))
        .collect()
}

fn check_if_prime(number: f64) -> Option<bool> {
    if number.fract() != 0.0 {
        return Some(false);
    }

    let num = number as u64;

    if num < 2 {
        return Some(false);
    }
    let has_divisor = (2..=num / 2).any(|x| num % x == 0);
    Some(!has_divisor)
    
    /*if num % 2 == 0 {
        return Some(false);
    }

    let sqrt_n = number.sqrt() as u64;
    for i in (3..=sqrt_n).step_by(2) {
        if num % i == 0 {
            return Some(false);
        }
    }
    Some(true)*/


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
        let result = check_if_prime(8.0).unwrap();
        assert_eq!(result, false);
        let result = check_if_prime(-4.0).unwrap();
        assert_eq!(result, false);
        let result = check_if_prime(13.0).unwrap();
        assert_eq!(result, true);
        let result = check_if_prime(144.0).unwrap();
        assert_eq!(result, false);
        let result = check_if_prime(7789.0).unwrap();
        assert_eq!(result, true);
        let result = check_if_prime(92946817.0).unwrap();
        assert_eq!(result, true);
    }
}
