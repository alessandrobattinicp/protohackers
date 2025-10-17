use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use regex::Regex;

/*
[Fri Oct 17 15:50:56 2025 UTC] [4notbogus.test] NOTE:PinkFrank450 joined the chat room
[Fri Oct 17 15:50:58 2025 UTC] [4notbogus.test] FAIL:message to 'GreenEdward629' was not correct
    (expected '[PinkFrank450] This is a product ID, not a Boguscoin: 7BCO8HH82v42s1r7NvGVkELN1r-OI50f7pcgUDwwjk2cCsTAE3rrI-1234'):
    [PinkFrank450] This is a product ID, not a Boguscoin: 7YWHMfk9JZe0LM0g1ZauHuiSxhI-OI50f7pcgUDwwjk2cCsTAE3rrI-1234
*/

fn main() {
    let listener = TcpListener::bind("0.0.0.0:5001").unwrap();

    for client in listener.incoming() {
        match client {
            Ok(client) => {
                handle_connection(client);
            }
            Err(e) => {
                println!("Tcp Error: {}", e);
            }
        }
    }
}

//user ---> proxy ---> budgetchat
fn handle_write(client: TcpStream, server: &mut TcpStream) {
    let mut client_reader = BufReader::new(&client);

    loop {
        let mut write_buffer = String::new();
        match client_reader.read_line(&mut write_buffer) {
            Ok(size) if size > 0 => {
                let new_message = check_boguscoin_address(write_buffer);
                println!("buffer write: {}", new_message);
                let _ = server.write(new_message.as_bytes());
                let _ = server.flush();
            }
            Ok(_) => {
                println!("Write: Buffer vuoto");
                let _ = client.shutdown(std::net::Shutdown::Both);
                let _ = server.shutdown(std::net::Shutdown::Both);
                break;
            }
            Err(e) => {
                println!("Write: Errore: {:?}", e);
                break;
            }
        };
    }
}

//user <--- proxy <--- budgetchat
fn handle_read(client: &mut TcpStream, server: TcpStream) {
    let mut server_reader = BufReader::new(server);
    loop {
        let mut read_buffer = String::new();
        match server_reader.read_line(&mut read_buffer) {
            Ok(size) if size > 0 => {
                let new_message = check_boguscoin_address(read_buffer);
                println!("buffer read: {}", new_message);
                let _ = client.write(new_message.as_bytes());
                let _ = client.flush();
            }
            Ok(_) => {
                println!("Read: Buffer vuoto");
                break;
            }
            Err(e) => {
                println!("Read: Errore: {:?}", e);
                break;
            }
        };
    }
}

fn handle_connection(client: TcpStream) {
    let server = TcpStream::connect("chat.protohackers.com:16963").unwrap();

    let client_write_clone = client.try_clone().unwrap();
    let mut client_read_clone = client.try_clone().unwrap();
    let mut server_write_clone = server.try_clone().unwrap();
    let server_read_clone = server.try_clone().unwrap();

    thread::spawn(move || handle_write(client_write_clone, &mut server_write_clone));
    thread::spawn(move || handle_read(&mut client_read_clone, server_read_clone));
}

//7[a-zA-Z0-9]{25,35}[\s $]
fn check_boguscoin_address(buffer: String) -> String {
    let re = Regex::new(r"7[a-zA-Z0-9]{25,35}[\s $]/s").unwrap();
    let new_buffer: String = re
        .replace_all(buffer.as_str(), "7YWHMfk9JZe0LM0g1ZauHuiSxhI")
        .into_owned();

    new_buffer
}

//
// fn check_boguscoin_address(buffer: String) -> String {
//     let re_s = Regex::new(r"7[a-zA-Z0-9]{25,34}\s").unwrap();
//     let re_ns = Regex::new(r"7[a-zA-Z0-9]{25,34}$").unwrap();

//     let new_buffer: String = re_s
//         .replace_all(buffer.as_str(), "7YWHMfk9JZe0LM0g1ZauHuiSxhI ")
//         .into_owned();

//     re_ns
//         .replace_all(new_buffer.as_str(), "7YWHMfk9JZe0LM0g1ZauHuiSxhI")
//         .into_owned()
// }
