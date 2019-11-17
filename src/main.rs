extern crate futures;
extern crate tokio;
extern crate websocket;

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};

use colored::*;

use futures::future::Future;
use futures::sink::Sink;
use futures::stream::Stream;
use futures::sync::mpsc;
use std::thread;
use websocket::result::WebSocketError;
use websocket::{ClientBuilder, OwnedMessage};
use serde::{Serialize, Deserialize};

type Timeout = Arc<Mutex<Instant>>;

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
    let server_config: ClientConfig = serde_json::from_str(&config_json[..]).expect("Error parsing config file!");
    return server_config;
}

fn start_pangobright() {
	println!("{}", "Starting pangobright.exe: Doom and gloom!".yellow());
    Command::new("cmd.exe")
    .args(&["/C", "pangobright.exe"])
    .spawn()
    .expect("Failed to kill pangobright!");
}
fn kill_pangobright() {
	println!("{}", "Killing pangobright.exe: Bright eyes and bushy tails!".yellow());
    Command::new("cmd.exe")
    .args(&["/C", "taskkill", "/IM", "pangobright.exe", "/F"])
    .output()
    .expect("Failed to kill pangobright!");
}

fn sleep_windows() {
	println!("{}", "Sleepy time".yellow());
    Command::new("cmd.exe")
    .args(&["/C", "shutdown", "/h"])
    .output()
    .expect("Failed to execute shutdown process!");
}

fn reset_ping_timeout(ta: Timeout) {
    let mut t = ta.lock().unwrap();
    *t = Instant::now();
}

fn check_ping_timeout(ta: Timeout) -> bool {
    let t = ta.lock().unwrap();
    return t.elapsed() > Duration::from_secs(15);
}

fn handle_msg(msg: String, ta: Timeout) -> String {
    let out = msg.clone().to_owned();
    let command: Vec<&str> = out.split(":").collect();
    if command.len() == 4 && command[0] == "alexaevent" && command[1] == "computer" {
        if command[2] == "power" && command[3] == "off" {
            sleep_windows();
		}
		if command[2] == "color" && command[3] == "dim" {
            start_pangobright();
		}
		if command[2] == "color" && command[3] == "bright" {
            kill_pangobright();
		}
    }
    reset_ping_timeout(ta);
    return msg;
}

fn ws_client() {
	// Load in config
    let client_config = load_config("config.json");
    let url = client_config.ws_url.clone() + "?room=" + &client_config.room[..] + "&key=" + &client_config.key[..];

	let mut runtime = tokio::runtime::current_thread::Builder::new()
		.build()
		.unwrap();


    let timeout: Timeout = Arc::new(Mutex::new(Instant::now()));
	let ping_check_timeout: Timeout = timeout.clone();
	
	// Communication channel to allow websocket listener to close when signaled by timeout watcher in separate thread
	let (timeout_sender, timeout_receiver) = mpsc::channel(0);

	let runner = ClientBuilder::new(&url.to_owned())
		.unwrap()
		//.add_protocol("rust-websocket")
		.async_connect_insecure()
		.and_then(|(duplex, _)| {
			println!("{}", "Connected to DAK server!".green());
			// Spawn ping timeout listener
			thread::spawn(move || {
				loop {
					if check_ping_timeout(ping_check_timeout.clone()) {
						let mut sink = timeout_sender.wait();
						sink.send(OwnedMessage::Close(None)).expect("Sending close msg to websocket client!");
						break;
					}
					thread::sleep(Duration::from_secs(30));
				}
			});
			let (sink, stream) = duplex.split();
			stream
				.filter_map(|message| {
					match message {
                        OwnedMessage::Text(msg) => {
							if msg != "ping" { println!("Received Message: {:?}", msg) };
                            Some(OwnedMessage::Text(handle_msg(msg, timeout.clone())))
                        },
						OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
						OwnedMessage::Ping(d) => Some(OwnedMessage::Pong(d)),
						_ => None,
					}
				})
				.select(timeout_receiver.map_err(|_| WebSocketError::NoDataAvailable))
				.forward(sink)
		});
	let _runtime_result = runtime.block_on(runner);
}

fn main() {
	loop {
		ws_client();
		println!("{}", "Disconnected from DAK server!".red());
		thread::sleep(Duration::from_secs(15));
	}
}