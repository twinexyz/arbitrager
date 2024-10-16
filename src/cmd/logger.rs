use tracing::Level;
use tracing_subscriber::EnvFilter;

pub fn logging(level: &String) {
    
    let log_level = match level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_env_filter(EnvFilter::new("arbitrager=debug"))
        .init();
}
