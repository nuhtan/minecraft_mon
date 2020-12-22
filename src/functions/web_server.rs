use std::{
    collections::VecDeque,
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    path::Path,
    sync::{mpsc::Sender, Arc, Mutex},
    thread,
};

// Import the functions from the same level file
use crate::functions::server_interactions;

pub fn handle_connections(
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
        "/data/players" => {
            server_interactions::get_players(player_count, player_count_max, players)
        }
        "/data/console" => server_interactions::get_console(chat),
        _ => {
            if request.len() > 11 as usize && &request[0..11] == "/data/send?" {
                server_interactions::send_command(&request[10..], web_sender)
            } else {
                if Path::new(
                    format!("../public/{}/{}", get_file_folder(request), &request[1..]).as_str(),
                )
                .exists()
                {
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
        _ => "images",
    }
}

fn get_file_contents(path: &str) -> String {
    fs::read_to_string(format!(
        "../public/{}/{}",
        get_file_folder(path),
        &path[1..]
    ))
    .expect(format!("Failed to read file: {}", path).as_str())
}

#[cfg(test)]
mod tests {
    #[test]
    fn sample() {
        assert_eq!(2 + 2, 4, "sample message");
    }
}
