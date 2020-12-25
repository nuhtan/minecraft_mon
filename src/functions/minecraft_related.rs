//!Module Description

/// Function Description
pub fn valid_username(name: &str) -> bool {
    for c in name.chars() {
        if !c.is_ascii_alphanumeric() && c != '_' {
            return false
        }
    }
    return true
}