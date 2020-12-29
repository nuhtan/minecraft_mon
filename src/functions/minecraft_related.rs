//!Module Description

use std::sync::{Arc, Mutex};

/// Function Description
pub fn valid_username(name: &str) -> bool {
    for c in name.chars() {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false;
        }
    }
    return true;
}

// Server output reading
pub fn server_output_scanning(
    line_content: &str,
    current_players: Arc<Mutex<Vec<String>>>,
    current_player_count: Arc<Mutex<u32>>,
    max_concurrent_player_count: Arc<Mutex<u32>>,
) {
    if &line_content[0..10] == "There are " {
        // list and latter should be removed and max should be modified.
        let list = &line_content[10..];
        let latter = &list[list.find(" ").unwrap() + 13..];
        let max = latter[0..latter.find(" ").unwrap()].parse::<u32>().unwrap();
        // Verify current players
        // Set/Update max player count
        let mut pc_max = max_concurrent_player_count.lock().unwrap();
        *pc_max = max;
    } else {
        match line_content.find(" ") {
            Some(loc) => {
                // Do length checks to avoid exceptions
                let name = &line_content[0..loc];
                // Player interaction
                if valid_username(name) {
                    // Player joining
                    if &line_content[loc + 1..line_content.len() - 1] == "joined the game" {
                        let mut players_current = current_players.lock().unwrap();
                        if !players_current.contains(&name.to_string()) {
                            players_current.push(name.to_string());
                            let mut pc = current_player_count.lock().unwrap();
                            *pc += 1;
                        }
                    // Player leaving
                    } else if &line_content[loc + 1..line_content.len() - 1] == "left the game" {
                        let mut players_current = current_players.lock().unwrap();
                        if players_current.contains(&name.to_string()) {
                            let loc = players_current
                                .iter()
                                .position(|look| name == look)
                                .unwrap();
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
}
