//! Crate wide documentation?
extern crate minecraft_monitor as mon;
use mon::functions::web_server::handle_connections;
use mon::functions::minecraft_related::*;

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{
    collections::VecDeque,
    process::{Command, Stdio},
};

/// The function that launches the application.
///
/// Launches a child process of a specified minecraft server.
/// Creates a thread to handle incoming connections.
/// Then creates a thread for the server input and a separate thread for the output.
///
/// The way that variables are shared across threads needs work,
/// potentially moving into a struct would help.
///
/// This section should contain details about how this function is launching everything
fn main() {
    env::set_current_dir(Path::new("./server")).unwrap();
    let (web_sender, web_receiver) = mpsc::channel::<String>();
    let player_count = Arc::new(Mutex::new(0));
    let player_count_max = Arc::new(Mutex::new(0));
    let players = Arc::new(Mutex::new(Vec::<String>::new()));
    let chat = Arc::new(Mutex::new(VecDeque::<(u32, String)>::new()));

    let mut child = Command::new("java")
        .args(&[
            "-Xms2G",
            "-Xmx4G",
            "-XX:+UseG1GC",
            "-jar",
            "paper-261.jar",
            "nogui",
        ])
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("Error starting server, refer to console for more details.");

    let mut server_out = BufReader::new(
        child
            .stdout
            .take()
            .expect("[Error] Failed to open server output"),
    );

    // Spawn a thread to handle incoming connections
    let read_chat = Arc::clone(&chat);
    let player_count_connection = player_count.clone();
    let players_connection = players.clone();
    let player_max_connection = player_count_max.clone();
    let connection_handle = thread::spawn(move || {
        handle_connections(
            Arc::clone(&player_count_connection),
            Arc::clone(&player_max_connection),
            Arc::clone(&players_connection),
            Arc::clone(&read_chat),
            web_sender.clone(),
        )
    });

    // Server interaction happens below
    let output_handle = thread::spawn(move || {
        let chat = Arc::clone(&chat);
        let current_players = Arc::clone(&players);
        let current_player_count = Arc::clone(&player_count);
        let max_concurrent_player_count = Arc::clone(&player_count_max);
        let mut line_num: u32 = 0;
        loop {
            let chat = Arc::clone(&chat);
            let mut buf = Vec::new();
            server_out.read_until(b'\n', &mut buf).unwrap();
            let line = String::from_utf8(buf).unwrap();
            let content = &line.clone()[17..];
            if line != "" {
                let mut term = chat.lock().unwrap();
                print!("\x1b[0;36m[Console]:\x1b[0m {}", line);
                term.push_front((line_num, line));
                while term.len() > 1000 {
                    term.pop_back();
                }
            }
            // Check if a player has joined
            if &content[0..10] == "There are " {
                let list = &content[10..];
                // let current = list[0..list.find(" ").unwrap()].parse::<u32>().unwrap();
                let latter = &list[list.find(" ").unwrap() + 13..];
                // println!("Latter: {:?}, List: {:?}", latter, list);
                let max = latter[0..latter.find(" ").unwrap()].parse::<u32>().unwrap();
                // Verify current players
                // Set/Update max player count
                let mut pc_max = max_concurrent_player_count.lock().unwrap();
                *pc_max = max;
            } else {
                match content.find(" ") {
                    Some(loc) => {
                        // Do length checks to avoid exceptions
                        let name = &content[0..loc];
                        // Player interaction
                        if valid_username(name) {
                            // Player joining
                            if &content[loc + 1..content.len() - 1] == "joined the game" {
                                let mut players_current = current_players.lock().unwrap();
                                if !players_current.contains(&name.to_string()) {
                                    players_current.push(name.to_string());
                                    let mut pc = current_player_count.lock().unwrap();
                                    *pc += 1;
                                }
                            // Player leaving
                            } else if &content[loc + 1..content.len() - 1] == "left the game" {
                                let mut players_current = current_players.lock().unwrap();
                                if players_current.contains(&name.to_string()) {
                                    let loc = players_current.iter().position(|look| name == look).unwrap();
                                    players_current.swap_remove(loc);
                                    let mut pc = current_player_count.lock().unwrap();
                                    *pc -= 1;
                                }
                            }
                        }
                    }
                    None => {
                        // There is no space and thus no way to match any of the possible 
                    }
                }
            }
            line_num += 1;
        }
    });

    let input_handle = thread::spawn(move || {
        loop {
            match web_receiver.recv_timeout(Duration::from_millis(200)) {
                Ok(mut cmd) => {
                    cmd = cmd + "\n";
                    print!("\x1b[0;35m[Command]:\x1b[0m {}", cmd);
                    {
                        let server_in = child.stdin.as_mut().unwrap();
                        server_in.write_all(cmd.as_bytes()).unwrap();
                    }
                }
                Err(_) => {
                    // Input is empty after time out, there is either no new input or the server is processing the next input.
                }
            }
        }
    });

    match connection_handle.join() {
        Ok(test) => test.unwrap(),
        Err(_) => {
            println!("Error in connection handle");
        }
    };
    output_handle.join().unwrap();
    input_handle.join().unwrap();
}
