pub mod cli;
pub mod config;
pub mod app;
pub mod diff;
pub mod exec;
pub mod input;
pub mod interval;
pub mod render;
pub mod screenshot;
pub mod terminal;

pub use crate::config::{ColorMode, Config, DifferencesMode};
