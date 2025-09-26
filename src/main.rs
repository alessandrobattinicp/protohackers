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
            Ok(streamz) => {
                thread::spawn(move || handle_connection(streamz, clients));
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
            println!("username: {:?}", username);
        }
        Ok(_) => {}
        Err(e) => {
            eprintln!("Read error: {}", e);
        }
    }

    {
        let mut list = clients.lock().unwrap();
        let mut names = Vec::<String>::new();

        for client in list.iter() {
            println!("Sono nel for per il Join del secondo utente");
            names.push(client.username.clone());
            let connection_stream = client.stream.try_clone().unwrap();

            let mut writer = BufWriter::new(&connection_stream);
            let _ = writer.write_all(format!("* {} has joined the room\n", username).as_bytes());
            match writer.flush() {
                Ok(_) => {
                    println!("Utente annunciato con:");
                    println!("* {} has joined the room\n", username);
                }
                Err(e) => {
                    println!("errore di flush {:?}", e);
                }
            }
        }

        let mut name_list = "* The room contains: ".to_string();
        name_list.push_str(names.join(",").as_str());
        name_list.push('\n');

        println!("Inizio annuncio");
        print!("{}", name_list);
        println!("Fine annuncio");

        let _ = writer.write_all(name_list.as_bytes());
        match writer.flush() {
            Ok(_) => {
                println!("Invio lista utenti");
            }
            Err(e) => {
                println!("errore di flush {:?}", e);
            }
        }

        /*

                          [Fri Sep 26 15:52:03 2025 UTC] [1client.test] NOTE:check starts
                [Fri Sep 26 15:52:03 2025 UTC] [1client.test] NOTE:connected to 93.66.32.222 port 5001
                [Fri Sep 26 15:52:06 2025 UTC] [1client.test] PASS
                [Fri Sep 26 15:52:07 2025 UTC] [2clients.test] NOTE:check starts
                [Fri Sep 26 15:52:07 2025 UTC] [2clients.test] NOTE:watchman connected to 93.66.32.222 port 5001
                [Fri Sep 26 15:52:07 2025 UTC] [2clients.test] NOTE:watchman joined the chat room
                [Fri Sep 26 15:52:07 2025 UTC] [2clients.test] NOTE:alice connected to 93.66.32.222 port 5001
                [Fri Sep 26 15:52:07 2025 UTC] [2clients.test] NOTE:bob connected to 93.66.32.222 port 5001
                [Fri Sep 26 15:52:07 2025 UTC] [2clients.test] NOTE:alice joined the chat room
        [Fri Sep 26 15:55:02 2025 UTC] [2clients.test] FAIL:message to 'watchman' was not correct (expected '[alice] I think I'm alone now'):  has joined the room

                            */

        username.truncate(username.len() - 1);
        list.push(Client { username, stream });
    }

    //3- avvisare gli altri client della connessione
    // inviare il messaggio "username joins"
    // inviare la lista dei presenti

    loop {
        let mut buffer: String = String::new();
        match reader.read_line(&mut buffer) {
            Ok(size) if size != 0 => {
                println!("ricevuto riga 115 {:?}", buffer);
            }
            Ok(_) => {
                // println!("ricevuto2 {:?}", buffer);
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
