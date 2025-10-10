use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

struct Client {
    username: String,  //lista di nomi
    stream: TcpStream, //lista di connessioni
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:5001").unwrap();
    let clients = Arc::new(Mutex::new(Vec::<Client>::new()));

    for stream in listener.incoming() {
        let clients = Arc::clone(&clients);
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_connection(stream, clients));
            }
            Err(e) => {
                println!("Tcp Error: {}", e);
            }
        }
    }
}

fn handle_connection(stream: TcpStream, clients: Arc<Mutex<Vec<Client>>>) {
    println!("Un client si è connesso");

    let connection_stream = stream.try_clone().unwrap();

    let mut reader = BufReader::new(&connection_stream);
    let mut writer = BufWriter::new(&connection_stream);

    // 1 - welcome
    let _ = writer.write_all("Welcome to budgetchat! What shall I call you?\n".as_bytes());
    match writer.flush() {
        Ok(_) => {
            println!("Nuova connessione ok");
        }
        Err(e) => {
            println!("errore di flush 43 {:?}", e);
        }
    }

    // 2 - username
    let mut username: String = String::new();
    match reader.read_line(&mut username) {
        Ok(size) if size >= 2 => {
            let is_valid = validate_username(username.as_bytes());
            if !is_valid {
                return;
            }
            println!("username: {:?}", username);
        }
        Ok(_) => {
            return;
        }
        Err(e) => {
            eprintln!("Read error: {}", e);
        }
    }

    {
        let mut list = clients.lock().unwrap();
        let mut names = Vec::<String>::new();

        for client in list.iter() {
            names.push(client.username.clone());
            let connection_stream = client.stream.try_clone().unwrap();

            let mut writer = BufWriter::new(&connection_stream);
            let _ = writer.write_all(format!("* room joined by {}", username).as_bytes());
            match writer.flush() {
                Ok(_) => {
                    println!("Utente annunciato con:");
                    println!("* {} has joined the room\n", username);
                }
                Err(e) => {
                    println!("{} errore di flush 81 {:?}", username, e);
                }
            }
        }

        let mut name_list = "* The room contains: ".to_string();
        name_list.push_str(names.join(",").as_str());
        name_list.push('\n');

        let _ = writer.write_all(name_list.as_bytes());
        match writer.flush() {
            Ok(_) => {
                println!("Invio lista utenti");
            }
            Err(e) => {
                println!("errore di flush 96 {:?}", e);
            }
        }

        username.truncate(username.len() - 1);
        list.push(Client {
            username: username.clone(),
            stream,
        });
    }

    loop {
        let mut buffer: String = String::new();
        match reader.read_line(&mut buffer) {
            Ok(size) if size > 0 => {
                let mut list = clients.lock().unwrap();
                for client in list.iter_mut() {
                    if client.username != username {
                        print!("{} > [{}] {}", client.username, username, buffer);
                        let _ = client
                            .stream
                            .write_all(format!("[{}] {}", username, buffer).as_bytes());
                    }
                }
            }
            Ok(_) => {
                println!("Utente {} disconnesso", username);
                let mut list = clients.lock().unwrap();

                let pos = list.iter().position(|x| x.username == username).unwrap();
                list.remove(pos);

                for client in list.iter_mut() {
                    let _ = client
                        .stream
                        .write_all(format!("* room left by {}\n", username).as_bytes());
                    let _ = writer.flush();
                }

                return;
            }
            Err(e) => {
                eprintln!("Read error: {}", e);
                break;
            }
        }
        // FAIL:server did not send 'ProtoBob51' the quit message for 'SlimyFrank231' within 10 seconds
        match writer.flush() {
            Ok(_) => {}
            Err(e) => {
                println!("errore di flush 139 {:?}", e);
                break;
            }
        }
    }
}

fn validate_username(str: &[u8]) -> bool {
    for c in str[..str.len() - 1].iter() {
        if !is_uppercase(c) && !is_lowercase(c) && !is_digit(c) {
            return false;
        }
    }

    true
}

fn is_backslashn(c: &u8) -> bool {
    *c == 10
}

fn is_uppercase(c: &u8) -> bool {
    *c >= 65 && *c <= 90
}

fn is_lowercase(c: &u8) -> bool {
    *c >= 97 && *c <= 122
}

fn is_digit(c: &u8) -> bool {
    *c >= 48 && *c <= 57
}
