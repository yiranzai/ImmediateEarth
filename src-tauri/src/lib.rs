use std::fs;
use std::path::PathBuf;
use chrono::{Utc, Timelike, Datelike};
use reqwest::blocking::Client;
use image::{ImageBuffer, imageops};
use tauri_plugin_shell::{ShellExt};
use serde::Serialize;
use serde_json::to_string;

// Learn more about Tauri commands at https://v2.tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    println!("Backend was called with an argument: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn update_earth_image() -> Result<String, String> {
    // Calculate time for Himawari-8 image (using UTC)
    let now = Utc::now() - chrono::Duration::minutes(30);
    
    let year = now.year().to_string();
    let month = format!("{:02}", now.month());
    let day = format!("{:02}", now.day());
    let hour = format!("{:02}", now.hour());
    let minute = format!("{:02}", (now.minute() / 10) * 10);
    
    // Create data directory using standard Rust fs
    let app_data_dir = std::env::var("LOCALAPPDATA").map_err(|_| "Failed to get LOCALAPPDATA environment variable")?;
    let mut base_path = PathBuf::from(app_data_dir);
    base_path.push("immediate_earth");
    fs::create_dir_all(&base_path).map_err(|e| format!("Failed to create base directory: {}", e))?;
    
    // Create tiles directory
    let mut tiles_path = base_path.clone();
    tiles_path.push("tiles");
    fs::create_dir_all(&tiles_path).map_err(|e| format!("Failed to create tiles directory: {}", e))?;
    
    // Image parameters
    let multiple = 4; // 4x4 grid for 2200x2200 resolution
    let tile_size = 550;
    let total_size = multiple * tile_size;
    
    // Create empty image
    let mut earth = ImageBuffer::new(total_size as u32, total_size as u32);
    let client = Client::new();
    
    // Download and stitch tiles
    for i in 0..multiple {
        for j in 0..multiple {
            let url = format!(
                "https://himawari.asia/img/D531106/{}d/550/{}/{}/{}/{}{}00_{}_{}.png",
                multiple, year, month, day, hour, minute, i, j
            );
            // 日志记录url
            println!("Downloading tile: {}", url);
            let response = client.get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
                .send()
                .map_err(|e| format!("Failed to download tile: {}", e))?;
            
            let tile_data = response.bytes().map_err(|e| format!("Failed to read tile data: {}", e))?;
            let tile = image::load_from_memory(&tile_data)
                .map_err(|e| format!("Failed to decode tile: {}", e))?
                .to_rgba8();
            
            // Save individual tile
            let tile_path = tiles_path.join(format!("tile_{}_{}.png", i, j));
            fs::write(&tile_path, &tile_data).map_err(|e| format!("Failed to save tile: {}", e))?;
            
            // Paste tile into position
            let x = i * tile_size;
            let y = j * tile_size;
            imageops::replace(&mut earth, &tile, x as i64, y as i64);
        }
    }
    
    // Save merged image
    let img_filename = format!("earth_{}{}{}_{}{}.png", year, month, day, hour, minute);
    let mut merged_img_path = base_path.clone();
    merged_img_path.push(&img_filename);
    earth.save(&merged_img_path).map_err(|e| format!("Failed to save merged image: {}", e))?;
    
    // Define a struct to hold the paths
    #[derive(Serialize)]
    struct ImagePaths {
        tiles_dir: String,
        merged_image: String
    }

    // Create instance with paths
    let image_paths = ImagePaths {
        tiles_dir: tiles_path.to_string_lossy().into_owned(),
        merged_image: merged_img_path.to_string_lossy().into_owned()
    };

    // Serialize to JSON
    to_string(&image_paths).map_err(|e| format!("Failed to serialize paths: {}", e))
}

#[tauri::command]
async fn set_wallpaper(app: tauri::AppHandle, path: &str) -> Result<String, String> {
    // Use PowerShell to set wallpaper on Windows
    let output = app
        .shell()
        .command("powershell")
        .args([
            "-Command",
            &format!(
                "Set-ItemProperty -Path \"HKCU:\\Control Panel\\Desktop\" -Name Wallpaper -Value \"{}\"; RUNDLL32.EXE user32.dll,UpdatePerUserSystemParameters /c /f",
                path.replace("\"", "\\\"")
            )
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok("Wallpaper set successfully".to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to set wallpaper: {}", stderr))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
                let window = tauri::Manager::get_webview_window(_app, "main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![greet, update_earth_image, set_wallpaper])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
