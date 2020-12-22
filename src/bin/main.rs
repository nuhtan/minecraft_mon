//! Crate wide documentation?
extern crate minecraft_monitor as mon;
use mon::functions::web_server::handle_connections;

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
/// Launches a child process of a specified minecraft server. Creates a thread to handle incoming connections. Then creates a thread for the server input and a seperate thread for the output.
/// The way that variables are shared accross threads needs work, potentially moving into a struct would help.
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
    let connection_handle = thread::spawn(move || {
        handle_connections (
            Arc::clone(&player_count),
            Arc::clone(&player_count_max),
            Arc::clone(&players),
            Arc::clone(&read_chat),
            web_sender.clone(),
        )
    });

    // Server interaction happens below
    let output_handle = thread::spawn(move || {
        let chat = Arc::clone(&chat);
        let mut line_num: u32 = 0;
        loop {
            let chat = Arc::clone(&chat);
            let mut buf = Vec::new();
            server_out.read_until(b'\n', &mut buf).unwrap();
            let line = String::from_utf8(buf).unwrap();
            // let content = &line.clone()[17..];
            if line != "" {
                let mut term = chat.lock().unwrap();
                print!("\x1b[0;36m[Console]:\x1b[0m {}", line);
                term.push_front((line_num, line));
                while term.len() > 1000 {
                    term.pop_back();
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
