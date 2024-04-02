pub fn format_log(file: &str, line: u32, message: &str) -> String {
    format!("{}:{} -> {}", file, line, message)
}
