//! Thin `#[path]` shim re-exporting [`wave_impl`] as `engine::wave` — kept
//! separate so the module's public path doesn't change if `wave_impl.rs` is
//! ever split further.

#[path = "wave_impl.rs"]
mod wave_impl;
pub use wave_impl::*;
