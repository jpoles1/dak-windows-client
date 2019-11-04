use colored::*;

extern crate tungstenite;
use std::fs;
use std::path::Path;
use std::process::Command;
use url::Url;
use tungstenite::{connect};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

// Structure of data expected in config.json
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ClientConfig {
    ws_url: String,
    room: String,
    key: String,
}

// Load in config settings from config.json in project root
fn load_config(filename: &str) -> ClientConfig {
    let config_json = fs::read_to_string(Path::new(filename)).expect("Error reading config file!");
    let server_config: ClientConfig = serde_json::from_str(&config_json[..]).unwrap();
    return server_config;
}

fn sleep_windows() {
    /*Command::new("cmd")
    .args(&["/C", "shutdown", "/h"])
    .output()
    .expect("Failed to execute shutdown process!");*/
    println!("Sleepy time");
}

fn main () {
    // Load in config
    let client_config = load_config("config.json");
    loop {
        let mut last_ping = Instant::now();
        // Connect to the url and call the closure
        let url = client_config.ws_url.clone() + "?room=" + &client_config.room[..] + "&key=" + &client_config.key[..];
        let (mut socket, response) = connect(Url::parse(&url[..]).unwrap()).expect("Can't connect to websocket server");

        println!("Connected to the server");
        println!("Response HTTP code: {}", response.code);
        println!("Response contains the following headers:");
        for &(ref header, _ /*value*/) in response.headers.iter() {
            println!("* {}", header);
        }

        loop {
            let msg = socket.read_message().expect("Error reading message");
            if msg.is_text() {
                let msg_txt = msg.into_text().unwrap();
                let command: Vec<&str> = msg_txt.split(":").collect();
                if command.len() == 4 && command[0] == "alexaevent" && command[1] == "computer" {
                    if command[2] == "power" && command[3] == "off" {
                        sleep_windows();
                    }
                }
            } else if msg.is_ping() {
                println!("Got ping!");
                last_ping = Instant::now();
            } else if msg.is_close() {
                println!("{}", "Got close message from server!".red());
            }
            if last_ping.elapsed() > Duration::from_secs(30) {
                println!("{}", "Timed out after 30 seconds without ping!".yellow());
                break;
            }
        }
        println!("{}", "Disconnected from socket!".yellow());
    }
    // socket.close(None);
}