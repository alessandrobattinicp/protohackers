use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[derive(Deserialize, Debug)]
struct Request {
    method: String,
    number: i32,
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
    let mut buffer: [u8; 512] = [0; 512];
    loop {
        match stream.read(&mut buffer).await {
            Ok(size) if size != 0 => {
                match serde_json::from_slice::<Request>(&buffer[..size]) {
                    Ok(request) => {
                        println!("{:?}",request);
                        if request.method != "isPrime" {
                            println!("richiesta malformata, {:?}", request);
                            let _ = stream.write("malformed".as_bytes()).await;
                            return;
                        }
                        let is_prime = check_if_prime(request.number).unwrap();
                        let response = Response {method:"isPrime".to_string(), prime: is_prime};
                        let _ = stream.write(serde_json::to_string(&response).unwrap().as_bytes()).await;
                    }
                    Err(_) => {
                        let _ = stream.write("malformed".as_bytes()).await;
                        return;
                    }
                };

                if let Err(e) = stream.write_all(&buffer[..size]).await {
                    eprintln!("Error writing to socket: {}", e);
                }
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

fn check_if_prime(number: i32) -> Option<bool> {
    if number < 2 {
        return Some(false);
    }
    let has_divisor = (2..=number / 2).any(|x| number % x == 0);
    Some(!has_divisor)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = check_if_prime(4).unwrap();
        assert_eq!(result, false);
        let result = check_if_prime(13).unwrap();
        assert_eq!(result, true);
        let result = check_if_prime(144).unwrap();
        assert_eq!(result, false);
        let result = check_if_prime(7789).unwrap();
        assert_eq!(result, true);
    }
}
