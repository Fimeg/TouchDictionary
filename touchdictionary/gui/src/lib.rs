#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::json;
use tauri::{command, generate_handler, Manager, WebviewWindow};
use tauri_plugin_opener::OpenerExt;
use touchdictionary_core::lookup;

#[command]
async fn run_lookup_command(query: String) -> Result<serde_json::Value, String> {
    println!(
        "[INFO] [touchdictionary] [gui] Lookup command invoked for: {}",
        query
    );

    match lookup::lookup(&query).await {
        Ok(result) => {
            println!(
                "[INFO] [touchdictionary] [gui] Successfully processed lookup for: {}",
                query
            );
            Ok(json!(result))
        }
        Err(e) => {
            println!(
                "[ERROR] [touchdictionary] [gui] Lookup failed for '{}': {}",
                query, e
            );
            Err(e)
        }
    }
}

#[command]
fn get_initial_query() -> Vec<String> {
    // Get command-line arguments passed to the app
    std::env::args().skip(1).collect()
}

#[command]
fn close_window(window: WebviewWindow) {
    println!("[INFO] [touchdictionary] [gui] Closing window");
    // Use the window handle to close/hide it
    if let Err(e) = window.hide() {
        println!("[ERROR] [touchdictionary] [gui] Failed to hide window: {}", e);
    }
}

#[command]
fn open_url(app: tauri::AppHandle, url: String) {
    println!("[INFO] [touchdictionary] [gui] Opening URL: {}", url);
    let _ = app.opener().open_url(url, None::<&str>);
}

pub fn run() {
    let context = tauri::generate_context!();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(generate_handler![run_lookup_command, get_initial_query, close_window, open_url])
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // Check if we have command-line arguments
            let args: Vec<String> = std::env::args().skip(1).collect();
            
            if !args.is_empty() {
                // Show the window immediately on startup with query
                let webview_window = app.get_webview_window("main").unwrap();
                webview_window.show().unwrap();
                webview_window.set_focus().unwrap();
                println!("[INFO] [touchdictionary] [gui] Window shown for query: {}", args.join(" "));
            } else {
                // No args - start hidden in background
                let webview_window = app.get_webview_window("main").unwrap();
                webview_window.hide().unwrap();
                println!("[INFO] [touchdictionary] [gui] No arguments provided, running in background mode");
            }
            
            Ok(())
        })
        .run(context)
        .expect("[ERROR] [touchdictionary] [gui] Failed to run Tauri application");
}
