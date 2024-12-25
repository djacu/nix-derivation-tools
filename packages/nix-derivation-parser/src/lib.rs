//! A module for parsing derivations from `.drv` files.
#![allow(clippy::implicit_return, reason = "clippy will ping pong on this forever.")]
#![allow(
    clippy::self_named_module_files,
    reason =
        "https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html?highlight=mod.rs#alternate-file-paths"
)]

pub mod derivations;
pub mod strings;
