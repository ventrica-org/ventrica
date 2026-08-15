use std::time;

pub fn make_temp_dir_name(name: &str) -> String {
    format!(
        "{}-{}",
        name,
        time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string()
    )
}
