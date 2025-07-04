use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

struct Clients {
    username: String,      //lista di nomi
    connections_list: i32, //lista di connessioni
}

// use std::sync::mpsc;
// use std::thread;

// fn main() {
//     let (tx, rx) = mpsc::channel();

//     thread::spawn(move || {
//         let val = String::from("hi");
//         tx.send(val).unwrap();
//     });

//     let received = rx.recv().unwrap();
//     println!("Got: {received}");
// }

fn main() {
    let listener = TcpListener::bind("0.0.0.0:5001").unwrap();
    let lista = Vec::<String>::new();

    for stream in listener.incoming() {
        match stream {
            Ok(streamz) => {
                thread::spawn(move || handle_connection(streamz, &lista));
            }
            Err(e) => {
                println!("Tcp Error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: TcpStream, clients_list: &Vec<String>) {
    println!("Un client si è connesso");

    let connection_stream = stream.try_clone().unwrap();
    let mut reader = BufReader::new(&connection_stream);
    let mut writer = BufWriter::new(&connection_stream);

    // 1 - welcome
    let _ = writer.write_all("Welcome to budgetchat! What shall I call you?\n".as_bytes());
    match writer.flush() {
        Ok(_) => {}
        Err(e) => {
            println!("errore di flush {:?}", e);
        }
    }

    // 2 - username
    let mut username: String = String::new();
    match reader.read_line(&mut username) {
        Ok(size) if size != 0 => {
            let is_valid = validate_username(username.as_bytes());
            if !is_valid {
                return;
            }
        }
        Ok(_) => {
            println!("ricevuto2 {:?}", username);
        }
        Err(e) => {
            eprintln!("Read error: {}", e);
        }
    }

    //3- avvisare gli altri client della connessione

    loop {
        let mut buffer: String = String::new();
        match reader.read_line(&mut buffer) {
            Ok(size) if size != 0 => {
                println!("ricevuto {:?}", buffer);
            }
            Ok(_) => {
                println!("ricevuto2 {:?}", buffer);
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

fn validate_username(str: &[u8]) -> bool {
    if str.len() < 1 {
        return false;
    }

    for c in str.to_owned() {
        if !is_uppercase(c) || !is_lowercase(c) || !is_digit(c) {
            return false;
        }
    }

    true
}

fn is_uppercase(c: u8) -> bool {
    c >= 65 && c <= 90
}

fn is_lowercase(c: u8) -> bool {
    c >= 97 && c <= 122
}

fn is_digit(c: u8) -> bool {
    c >= 48 && c <= 57
}
