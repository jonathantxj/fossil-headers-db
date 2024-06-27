pub fn strip_hex_string(hex_string: &str) -> String {
    hex_string.trim_start_matches("0x").to_string()
}

pub fn convert_hex_string_to_i64(hex_string: &str) -> i64 {
    i64::from_str_radix(&strip_hex_string(hex_string), 16).unwrap()
}
