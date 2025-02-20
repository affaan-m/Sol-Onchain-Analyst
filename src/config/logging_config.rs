use tracing::Level;

pub fn get_log_level() -> Level {
    match std::env::var("RUST_LOG") {
        Ok(val) => match val.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        },
        Err(_) => Level::INFO,
    }
}
