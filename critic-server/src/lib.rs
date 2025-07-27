//! Things that only run on the server (interacting with the servers fs and external APIs)
//!
//! Also contains some axum routes that are static or directly linked to external APIs (like the
//! oauth flow).
pub mod auth;
pub mod config;
pub mod db;
pub mod github;
pub mod minification;
pub mod signal_handler;
pub mod static_files;
pub mod upload;
