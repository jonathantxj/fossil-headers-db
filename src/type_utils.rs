use hex::decode;

pub fn strip_hex_string(hex_string: &String) -> String {
    hex_string.trim_start_matches("0x").to_string()
}

pub fn convert_hex_string_to_i64(hex_string: &String) -> i64 {
    i64::from_str_radix(&strip_hex_string(&hex_string), 16).unwrap()
}

pub fn convert_hex_string_to_bytes(hex_string: &String) -> Vec<u8> {
    decode(&strip_hex_string(&hex_string)).unwrap()
}

pub fn option_fn_handler<T, F, U>(f: F, value: Option<T>) -> Option<U>
where
    F: Fn(&T) -> U,
{
    match value {
        Some(x) => Some(f(&x)),
        None => None,
    }
}
