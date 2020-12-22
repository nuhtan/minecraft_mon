use std::{
    collections::VecDeque,
    sync::{mpsc::Sender, Arc, Mutex},
};

/// Returns a String, in JSON format, of the current player data from the minecraft server
///
/// This function safely reads status variables from the minecraft server.
/// All players are looped over to populate a string with JSON content.
/// These variables are updated in [`main`] when players join or leave the server.
///
/// # Examples
///
///```
///use std::sync::Arc;
///use std::sync::Mutex;
///use minecraft_monitor::functions::server_interactions::get_players;
///
///let count = 2;
///let max = 20;
///let players = vec![String::from("player1"), String::from("player2")];
///
///println!("{}", get_players(Arc::new(Mutex::new(count)), Arc::new(Mutex::new(max)), Arc::new(Mutex::new(players))));
///```
pub fn get_players(
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
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: Close\r\n\r\n{}",
        data
    )
}

pub fn get_console(chat: Arc<Mutex<VecDeque<(u32, String)>>>) -> String {
    let chat = chat.lock().unwrap();
    let mut log = format!("{{\"chat\": {{\n");
    for line in chat.iter() {
        log.push_str(
            format!(
                "\"{}\":\"{}\",\n",
                line.0,
                line.1.replace("\n", "").replace("\"", "\\\"")
            )
            .as_str(),
        );
    }
    log.push_str("}}");
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: Close\r\n\r\n{}",
        log
    )
}

fn query_string(query: &str) -> String {
    query[1..].replace("_", " ").to_string()
}

pub fn send_command(command: &str, web_sender: Sender<String>) -> String {
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

#[cfg(test)]
mod tests {
    #[test]
    fn sample() {
        assert_eq!(2 + 2, 4, "sample message");
    }
}