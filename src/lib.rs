//! # cargo-projects
//! 
//! A "Work-In-Progress" 
//! - and incomplete 
//! - and untested 
//! - and unstable
//! - CLI tool for managing Rust projects - list, track, and clean up your local cargo projects.
//! 
//! ## Overview
//! `cargo-projects` does the following:
//! - Tracks your Rust projects and monitors their disk usage
//! - Clean up listed projects easily
//! - Watches directories that you specify to track for Rust projects
//! 
//! 


pub mod types;
pub mod commands;
pub mod output;
pub mod infrastructure;
pub mod services;
pub mod repositories; 
