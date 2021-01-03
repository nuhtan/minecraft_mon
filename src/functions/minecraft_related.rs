//!Module Description
use super::shared_data;

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
pub fn server_output_scanning(line_content: &str, data: shared_data::ServerSharedData) {
    if &line_content[0..10] == "There are " {
        // list and latter should be removed and max should be modified.
        let list = &line_content[10..];
        let latter = &list[list.find(" ").unwrap() + 13..];
        let max = latter[0..latter.find(" ").unwrap()].parse::<u32>().unwrap();
        // Verify current players
        // Set/Update max player count
        let mut pc_max = data.max_player_count.lock().unwrap(); // FIXME try_lock?
        *pc_max = max;
    } else if &line_content[0..6] == "Done (" {
        let exc = line_content.find(")!").unwrap();
        if &line_content[exc..] == ")! For help, type \"help\"\n" {
            let mut state = data.mcserver_state.lock().unwrap();
            *state = shared_data::MinecraftServerState::Running;
        }
    } else if &line_content[..] == "Closing Server\n" {
        let mut state = data.mcserver_state.lock().unwrap();
        *state = shared_data::MinecraftServerState::Off;
    } else {
        match line_content.find(" ") {
            Some(loc) => {
                // Do length checks to avoid exceptions
                let name = &line_content[0..loc];
                // Player interaction
                if valid_username(name) {
                    // TODO Server starting and closing
                    // Player joining
                    if &line_content[loc + 1..line_content.len() - 1] == "joined the game" {
                        let mut players_current = data.current_players.lock().unwrap(); // FIXME try_lock?
                        if !players_current.contains(&name.to_string()) {
                            players_current.push(name.to_string());
                            let mut pc = data.current_player_count.lock().unwrap(); // FIXME try_lock?
                            *pc += 1;
                        }
                    // Player leaving
                    } else if &line_content[loc + 1..line_content.len() - 1] == "left the game" {
                        let mut players_current = data.current_players.lock().unwrap(); // FIXME try_lock?
                        if players_current.contains(&name.to_string()) {
                            let loc = players_current
                                .iter()
                                .position(|look| name == look)
                                .unwrap();
                            players_current.swap_remove(loc);
                            let mut pc = data.current_player_count.lock().unwrap(); // FIXME try_lock?
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
