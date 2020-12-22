use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{
    collections::VecDeque,
    process::{Command, Stdio},
};
use std::{fs, sync::mpsc::Sender, time::Duration};
use std::env;

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
        handle_connections(
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
        Ok(test) => {test.unwrap()}
        Err(_) => {
            println!("Error in connection handle");
        }
    };
    output_handle.join().unwrap();
    input_handle.join().unwrap();
}

fn handle_connections(
    player_count: Arc<Mutex<i32>>,
    player_count_max: Arc<Mutex<i32>>,
    players: Arc<Mutex<Vec<String>>>,
    chat: Arc<Mutex<VecDeque<(u32, String)>>>,
    web_sender: Sender<String>,
) -> std::io::Result<()> {
    loop {
        let listener = TcpListener::bind("0.0.0.0:8000")?;
        let pc = player_count.clone();
        let pcm = player_count_max.clone();
        let p = players.clone();
        let c = chat.clone();
        let sender = web_sender.clone();
        // For each request create a thread to parse request and send contents
        for stream in listener.incoming() {
            let pc = pc.clone();
            let pcm = pcm.clone();
            let p = p.clone();
            let c = c.clone();
            let sender = sender.clone();
            thread::spawn(move || {
                let pc = pc.clone();
                let pcm = pcm.clone();
                let p = p.clone();
                let c = c.clone();
                let sender = sender.clone();
                let mut stream = stream.unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                match line.find("/") {
                    // Every proper request line should contain a forward slash
                    Some(start) => {
                        let request = &line[start..line.find("HTTP").unwrap() - 1];
                        stream
                            .write_all(generate_response(request, pc, pcm, p, c, sender).as_bytes())
                            .unwrap();
                    }
                    None => {
                        println!("\x1b[0;33m[Request]:\x1b[0m Empty Request Recieved");
                    }
                }
            });
        }
    }
}

fn generate_response(
    request: &str,
    player_count: Arc<Mutex<i32>>,
    player_count_max: Arc<Mutex<i32>>,
    players: Arc<Mutex<Vec<String>>>,
    chat: Arc<Mutex<VecDeque<(u32, String)>>>,
    web_sender: Sender<String>,
) -> String {
    let default_http_header = "HTTP/1.1 200 OK\r\nConnection: Close\r\nContent-Type:";
    let headers404 = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nConnection: Close";
    println!("\x1b[0;33m[Request]:\x1b[0m {}", request);
    match request {
        "/" => format!(
            "{} text/html\r\n\r\n{}",
            default_http_header,
            get_file_contents("/home.html")
        ),
        "/data/players" => get_players(player_count, player_count_max, players),
        "/data/console" => get_console(chat),
        _ => {
            if request.len() > 11 as usize && &request[0..11] == "/data/send?" {
                send_command(&request[10..], web_sender)
            } else {
                if Path::new(format!("../public/{}/{}", get_file_folder(request), &request[1..]).as_str()).exists() {
                    format!(
                        "{} {}\r\n\r\n{}",
                        default_http_header,
                        get_file_type(request),
                        get_file_contents(request)
                    )
                } else {
                    headers404.to_string()
                }
            }
        }
    }
}

fn get_file_type(path: &str) -> &str {
    let ext = &path[path.find(".").unwrap()..];
    match ext {
        ".html" => "text/html",
        ".png" => "image/png",
        ".jpg" | ".jpeg" => "image/jpeg",
        ".gif" => "image/gif",
        ".ico" => "image/x-icon",
        _ => "text/plain",
    }
}

fn get_file_folder(path: &str) -> &str {
    let ext = &path[path.find(".").unwrap()..];
    // println!("{} - {}", path, ext);
    match ext {
        ".html" => "html",
        ".css" => "css",
        ".js" => "javascript",
        _ => "images"
    }
}

fn get_file_contents(path: &str) -> String {
    fs::read_to_string(format!("../public/{}/{}", get_file_folder(path), &path[1..])).expect(format!("Failed to read file: {}", path).as_str())
}

fn get_players(
    player_count: Arc<Mutex<i32>>,
    player_count_max: Arc<Mutex<i32>>,
    players: Arc<Mutex<Vec<String>>>,
) -> String {
    let pc = player_count.lock().unwrap();
    let pcm = player_count_max.lock().unwrap();
    let p = players.lock().unwrap();
    let mut data = format!(
        "{{\"playerCount\": \"{}\", \"playerCountMax\": \"{}\", \"player\": [",
        pc, pcm
    );
    for player in p.iter() {
        data.push_str((*player).as_str());
        data.push_str(", ");
    }
    data.push_str("]}");
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: Close\r\n\r\n{}", data)
}

fn get_console(chat: Arc<Mutex<VecDeque<(u32, String)>>>) -> String {
    let chat = chat.lock().unwrap();
    let mut log = format!("{{\"chat\": {{\n");
    for line in chat.iter() {
        log.push_str(format!("\"{}\":\"{}\",\n", line.0, line.1.replace("\n", "").replace("\"", "\\\"")).as_str());
    }
    log.push_str("}}");
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: Close\r\n\r\n{}", log)
}

fn query_string(query: &str) -> String {
    query[1..].replace("_", " ").to_string()
}

fn send_command(command: &str, web_sender: Sender<String>) -> String {
    match web_sender.send(query_string(command)) {
        Ok(_) => {
            "HTTP/1.1 201 Created\r\nContent-Type: text/plain\r\nConnection: Close".to_string()
        }
        Err(_) => {
            "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\nConnection: Close"
                .to_string()
        }
    }
}
