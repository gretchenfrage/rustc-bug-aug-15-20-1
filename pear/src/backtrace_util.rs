
use std::env;
use backtrace::Backtrace;

pub fn is_backtrace_enabled() -> bool {
    env::var("RUST_BACKTRACE")
        .map(|val| val != "0")
        .unwrap_or(false)
}

pub fn capture_backtrace_if_enabled() -> Option<Backtrace> {
    if is_backtrace_enabled() {
        Some(Backtrace::new())
    } else {
        None
    }
}