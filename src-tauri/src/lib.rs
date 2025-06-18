use chrono::{Datelike, Timelike, Utc};
use image::{self, GenericImageView};
use reqwest::blocking::Client;
use serde::{Serialize, Deserialize};
use serde_json::to_string;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tauri::tray::{TrayIconBuilder};
use tauri::menu::{Menu, MenuItem, MenuItemBuilder, SubmenuBuilder, MenuBuilder};
use tauri_plugin_opener::OpenerExt;
mod wallpaper;

// Learn more about Tauri commands at https://v2.tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    println!("Backend was called with an argument: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn update_earth_image(app: tauri::AppHandle) -> Result<String, String> {
    // Calculate time for Himawari-8 image (using UTC)
    let now = Utc::now() - chrono::Duration::minutes(30);

    let year = now.year().to_string();
    let month = format!("{:02}", now.month());
    let day = format!("{:02}", now.day());
    let hour = format!("{:02}", now.hour());
    let minute = format!("{:02}", (now.minute() / 10) * 10);

    // 使用 Tauri 的路径解析器创建数据目录，以实现跨平台兼容
    let app_data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("无法获取应用本地数据目录: {}", e))?;
    let mut base_path = PathBuf::from(app_data_dir);
    base_path.push("immediate_earth");
    fs::create_dir_all(&base_path).map_err(|e| format!("无法创建基础目录: {}", e))?;

    // Create tiles directory
    let mut tiles_path = base_path.clone();
    tiles_path.push("tiles");
    fs::create_dir_all(&tiles_path)
        .map_err(|e| format!("Failed to create tiles directory: {}", e))?;

    // Image parameters
    let multiple = 4; // 4x4 grid for 2200x2200 resolution
    let tile_size = 550;
    let total_size = multiple * tile_size;

    // Create empty image
    let mut earth = image::ImageBuffer::new(total_size as u32, total_size as u32);
    let client = Client::new();

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

            let tile_data = response
                .bytes()
                .map_err(|e| format!("Failed to read tile data: {}", e))?;
            if tile_data.len() == 2834 {
                return Err(format!("本次爬取失败，tile({},{})大小异常", i, j));
            }
            let tile = image::load_from_memory(&tile_data)
                .map_err(|e| format!("Failed to decode tile: {}", e))?
                .to_rgba8();

            // Save individual tile
            let tile_path = tiles_path.join(format!("tile_{}_{}.png", i, j));
            fs::write(&tile_path, &tile_data).map_err(|e| format!("Failed to save tile: {}", e))?;

            // Paste tile into position
            let x = i * tile_size;
            let y = j * tile_size;
            image::imageops::replace(&mut earth, &tile, x as i64, y as i64);
        }
    }

    // Save merged image
    let img_filename = format!("earth_{}{}{}_{}{}.png", year, month, day, hour, minute);
    let mut merged_img_path = base_path.clone();
    merged_img_path.push(&img_filename);
    earth
        .save(&merged_img_path)
        .map_err(|e| format!("Failed to save merged image: {}", e))?;

    // 1. 生成 black 后缀路径
    let black_img_path = add_suffix_to_filename(&merged_img_path, "_black");

    // 2. 读取原始大图，加黑边
    let path = Path::new(&merged_img_path);
    let img = image::open(path)
        .map_err(|e| format!("读取拼接后大图失败: {}", e))?;
    let (orig_img_width, orig_img_height) = img.dimensions();
    let img_width = orig_img_width;
    let img_height = orig_img_height;

    // black_border 基于原始宽度
    let black_border = (orig_img_width as f32 / 10.0).round() as u32;

    let new_w = img_width + black_border * 2;
    let new_h = img_height + black_border * 2;
    let mut canvas = image::RgbaImage::from_pixel(new_w, new_h, image::Rgba([0, 0, 0, 255]));
    image::imageops::replace(&mut canvas, &img.to_rgba8(), black_border as i64, black_border as i64);

    // 3. 保存 black 图
    canvas.save(&black_img_path)
        .map_err(|e| format!("保存加黑边大图失败: {}", e))?;
    println!("【地球大图】加黑边后图片已保存: {}", black_img_path.to_string_lossy());

    // 4. 返回 black 图路径
    #[derive(Serialize)]
    struct ImagePaths {
        tiles_dir: String,
        merged_image: String,      // 原始大图
        black_image: String,       // 带黑边大图
    }
    let image_paths = ImagePaths {
        tiles_dir: tiles_path.to_string_lossy().into_owned(),
        merged_image: merged_img_path.to_string_lossy().into_owned(),
        black_image: black_img_path.to_string_lossy().into_owned(),
    };
    to_string(&image_paths).map_err(|e| format!("Failed to serialize paths: {}", e))
}

/// 辅助函数：为文件名添加后缀，保留原扩展名
pub fn add_suffix_to_filename(path: &Path, suffix: &str) -> PathBuf {
    let mut new_path = path.to_path_buf();
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().unwrap_or_default().to_string_lossy();
    let new_name = if ext.is_empty() {
        format!("{}{}", stem, suffix)
    } else {
        format!("{}{}.{}", stem, suffix, ext)
    };
    new_path.set_file_name(new_name);
    new_path
} 

#[tauri::command]
fn get_image_dir(app: tauri::AppHandle) -> Result<String, String> {
    let app_data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("无法获取应用本地数据目录: {}", e))?;
    let mut base_path = std::path::PathBuf::from(app_data_dir);
    base_path.push("immediate_earth");
    Ok(base_path.to_string_lossy().to_string())
}

/// 只保留今天的图片，其余全部删除
#[tauri::command]
fn clean_old_images(app: tauri::AppHandle) -> Result<(), String> {
    use chrono::Datelike;
    // 获取图片保存目录
    let app_data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("无法获取应用本地数据目录: {}", e))?;
    let mut base_path = std::path::PathBuf::from(app_data_dir);
    base_path.push("immediate_earth");

    // 获取今天日期字符串，格式：yyyyMMdd
    let now = chrono::Utc::now();
    let today_str = format!("{:04}{:02}{:02}", now.year(), now.month(), now.day());

    // 遍历 immediate_earth 目录下所有文件
    if let Ok(entries) = std::fs::read_dir(&base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            // 跳过目录（如 tiles）
            if path.is_dir() { continue; }
            // 只处理 png 文件
            if let Some(ext) = path.extension() {
                if ext != "png" { continue; }
            } else {
                continue;
            }
            // 文件名中包含今天日期则保留，否则删除
            let fname = path.file_name().unwrap_or_default().to_string_lossy();
            if !fname.contains(&today_str) {
                let _ = std::fs::remove_file(&path);
                println!("【前端定时清理】已删除旧图片: {}", path.to_string_lossy());
            }
        }
    }
    Ok(())
}

/// 通过 OpenWeather API 获取天气信息，key 由前端传递
#[tauri::command]
fn get_weather(city: String, key: String) -> Result<String, String> {
    // 直接用前端传来的 key
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric&lang=zh_cn",
        city, key
    );
    let client = Client::new();
    let resp = client.get(&url)
        .send()
        .map_err(|e| format!("请求OpenWeather失败: {}", e))?
        .text()
        .map_err(|e| format!("读取OpenWeather响应失败: {}", e))?;

    #[derive(Deserialize)]
    struct WeatherResp {
        weather: Vec<WeatherDesc>,
        main: WeatherMain,
        name: String,
    }
    #[derive(Deserialize)]
    struct WeatherDesc {
        description: String,
    }
    #[derive(Deserialize)]
    struct WeatherMain {
        temp: f32,
        humidity: u8,
    }

    let weather: WeatherResp = serde_json::from_str(&resp)
        .map_err(|e| format!("解析OpenWeather响应失败: {}", e))?;
    if !weather.weather.is_empty() {
        let desc = &weather.weather[0].description;
        let temp = weather.main.temp;
        let humidity = weather.main.humidity;
        let city = &weather.name;
        Ok(format!(
            "{} {}°C 湿度{}% ({})",
            desc, temp.round(), humidity, city
        ))
    } else {
        Err("未获取到天气信息".to_string())
    }
}

// 新增：获取所有显示器信息的命令
#[tauri::command]
fn get_all_monitors(app: AppHandle) -> Result<String, String> {
    let monitors = app.available_monitors()
        .map_err(|e| format!("获取显示器信息失败: {}", e))?;
    let primary = app.primary_monitor()
        .map_err(|e| format!("获取主屏幕失败: {}", e))?;
    #[derive(Serialize)]
    struct MonitorInfo {
        name: String,
        position: (i32, i32),
        size: (u32, u32),
        scale_factor: f64,
        is_primary: bool,
    }
    let monitor_infos: Vec<MonitorInfo> = monitors.iter().map(|m| {
        let is_primary = if let Some(ref p) = primary {
            m.name() == p.name() &&
            m.position().x == p.position().x &&
            m.position().y == p.position().y &&
            m.size().width == p.size().width &&
            m.size().height == p.size().height &&
            (m.scale_factor() - p.scale_factor()).abs() < 0.0001
        } else {
            false
        };
        MonitorInfo {
            name: m.name().map_or("Unknown".to_string(), |s| s.to_string()),
            position: (m.position().x, m.position().y),
            size: (m.size().width, m.size().height),
            scale_factor: m.scale_factor(),
            is_primary,
        }
    }).collect();
    serde_json::to_string(&monitor_infos)
        .map_err(|e| format!("序列化显示器信息失败: {}", e))
}

// 修改：为所有显示器设置壁纸
#[tauri::command]
async fn set_wallpaper_for_all_monitors(
    app: AppHandle,
    image_path: String,
    platform: String,
    monitor_indexes: Option<Vec<usize>>,
) -> Result<String, String> {
    let monitors = app.available_monitors()
        .map_err(|e| format!("获取显示器信息失败: {}", e))?;
    // 打印所有屏幕详细信息
    println!("【所有屏幕信息】");
    for (idx, m) in monitors.iter().enumerate() {
        println!(
            "屏幕{}: name={:?}, position=({},{}) size=({}x{}) scale_factor={}",
            idx,
            m.name(),
            m.position().x, m.position().y,
            m.size().width, m.size().height,
            m.scale_factor()
        );
    }
    // 处理要设置的屏幕索引
    let target_indexes: Vec<usize> = if let Some(ref idxs) = monitor_indexes {
        if idxs.is_empty() {
            (0..monitors.len()).collect()
        } else {
            idxs.clone()
        }
    } else {
        (0..monitors.len()).collect()
    };
    let mut results = Vec::new();
    for &index in &target_indexes {
        if let Some(monitor) = monitors.get(index) {
            println!("【设置壁纸】正在处理显示器 {}: {}x{}", 
                index, monitor.size().width, monitor.size().height);
            let cropped_path = wallpaper::crop_image_for_monitor(&app, &image_path, index).await?;
            wallpaper::set_wallpaper_for_monitor(cropped_path.clone(), platform.clone(), index).await?;
            results.push(cropped_path);
        } else {
            println!("警告：索引{}的屏幕不存在，跳过", index);
        }
    }
    Ok(serde_json::to_string(&results)
        .map_err(|e| format!("序列化结果失败: {}", e))?)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // 系统托盘
            let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "显示窗口", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    _ => {
                        println!("menu item {:?} not handled", event.id);
                    }
                })
                .build(app)?;

            // 注册全局菜单事件处理（托盘菜单项点击也会触发这里）
            let app_handle = app.handle().clone();
            app_handle.on_menu_event(move |app, event| {
                match event.id().0.as_str() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => app.exit(0),
                    _ => {
                        println!("menu item {:?} not handled", event.id);
                    }
                }
            });

            // 监听窗口关闭，实现最小化到托盘
            let window = app.get_webview_window("main").unwrap();
            window.clone().on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    #[cfg(not(target_os = "macos"))] {
                        let win = app_handle.get_webview_window("main").unwrap();
                        let _ = win.hide();
                    }
                    #[cfg(target_os = "macos")] {
                        tauri::AppHandle::hide(&window.app_handle()).unwrap();
                    }
                    api.prevent_close();
                }
            });


            // 窗口菜单
            let cust_menu = MenuItemBuilder::with_id("Tauri Doc", "Tauri Doc").build(app)?;
            let main_menu = SubmenuBuilder::with_id(app, "main_menu", "主菜单")
                .items(&[&cust_menu])
                .build()?;
            let menu = MenuBuilder::new(app)
                .items(&[&main_menu])
                .build()?;
            app.set_menu(menu)?;
            app.on_menu_event(move |app, event| {
                if event.id() == cust_menu.id() {
                    let _ = app.opener().open_url("https://github.com/tauri-apps/tauri", None::<&str>);
                }
            });



            #[cfg(debug_assertions)] // 仅在开发模式下打开 devtools
            {
                let window = app.get_webview_window("main").unwrap();
                let _ = window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            update_earth_image,
            get_image_dir,
            clean_old_images,
            get_weather,
            get_all_monitors,
            set_wallpaper_for_all_monitors
        ])
        .run(tauri::generate_context!())
        .expect("启动失败");
}
