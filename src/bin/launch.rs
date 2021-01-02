//! Crate wide documentation?
extern crate minecraft_monitor as mon;
use mon::functions::configuration::determine_config;
use mon::functions::minecraft_related::*;
use mon::functions::shared_data::*;
use mon::functions::web_server::handle_connections;

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

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
    match determine_config(env::args().collect()) {
        Ok(work) => {
            let (
                address,
                port,
                web_index,
                root_location,
                jar_name,
                gen_args,
                min_ram,
                max_ram,
                web_log,
                verbosity,
            ) = work;
        }
        Err(err) => {
            println!("error found: {}", err)
        }
    }

    return;

    // TODO Refactor functions
    env::set_current_dir(Path::new("./server")).unwrap();
    let (web_sender, web_receiver) = mpsc::channel::<String>();
    let shared_data = ServerSharedData::new();

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
    let shared_data_connection = shared_data.clone();
    let connection_handle =
        thread::spawn(move || handle_connections(shared_data_connection, web_sender.clone()));

    // Server interaction happens below
    let shared_data_output = shared_data.clone();
    let output_handle = thread::spawn(move || {
        let data = shared_data_output.clone();
        let mut line_num: u32 = 0;
        loop {
            let chat = data.server_output.clone();
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
            server_output_scanning(content, data.clone());
            line_num += 1;
        }
    });

    let input_handle = thread::spawn(move || {
        loop {
            // Sleeping per the tick rate, this might be slightly extreme for the purposes of this application
            match web_receiver.recv_timeout(Duration::from_millis(50)) {
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

    // Can this be changed?
    match connection_handle.join() {
        Ok(test) => test.unwrap(),
        Err(_) => {
            println!("Error in connection handle");
        }
    };
    output_handle.join().unwrap();
    input_handle.join().unwrap();
}
