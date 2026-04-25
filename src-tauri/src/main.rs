// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Entry point for the Longchamp Budget application.

/// Main function that starts the Tauri application.
fn main() {
    longchamp_budget_lib::run()
}
