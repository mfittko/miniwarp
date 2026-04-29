pub use warp_core::features::*;

pub const TERMINAL_CORE_MODE: bool = true;

pub fn is_terminal_core_mode() -> bool {
    TERMINAL_CORE_MODE
}
