//! REST API server for executing WDL workflows.
//!
//! Sprocket Server provides a high-performance API for submitting and managing
//! concurrent WDL workflow executions with persistent storage.

pub mod api;
pub mod config;
pub mod db;
pub mod manager;
pub mod names;
pub mod server;

pub use config::Config;
pub use db::Database;
pub use server::run;
