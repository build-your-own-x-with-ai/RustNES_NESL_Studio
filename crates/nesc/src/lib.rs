//! Library side of `nesc`: the shared JSON payloads and the editor web server.

pub mod json;

#[cfg(not(target_arch = "wasm32"))]
pub mod server;
