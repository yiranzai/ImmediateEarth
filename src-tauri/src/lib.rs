use chrono::{Datelike, Timelike, Utc, Duration};
use image::{self, GenericImageView, Rgba, RgbaImage, imageops, DynamicImage};
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::to_string;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Manager};

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

            let tile_data = response
                .bytes()
                .map_err(|e| format!("Failed to read tile data: {}", e))?;
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

/// 设置系统壁纸的跨平台命令
/// 参数 image_path 必须为绝对路径，platform 由前端传递
#[tauri::command]
async fn set_wallpaper(image_path: String, platform: String) -> Result<(), String> {
    println!("【设置壁纸】收到请求，图片路径: {}, 平台: {}", image_path, platform);

    // 检查路径是否为绝对路径
    if !std::path::Path::new(&image_path).is_absolute() {
        println!("【设置壁纸】图片路径不是绝对路径: {}", image_path);
        return Err("图片路径必须为绝对路径".into());
    } else {
        println!("【设置壁纸】图片路径校验通过");
    }

    match platform.as_str() {
        "windows" => {
            let cmd = format!(
                "Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; \
                public class Wallpaper {{ [DllImport(\"user32.dll\", CharSet=CharSet.Auto)] \
                public static extern int SystemParametersInfo(int uAction, int uParam, String lpvParam, int fuWinIni); }}'; \
                [Wallpaper]::SystemParametersInfo(20, 0, '{}', 3)",
                image_path
            );
            println!("【设置壁纸】Windows 平台，执行 PowerShell 命令: {}", cmd);
            let result = Command::new("powershell")
                .args(&["-Command", &cmd])
                .spawn();
            match result {
                Ok(_) => println!("【设置壁纸】Windows 壁纸设置命令已执行"),
                Err(e) => {
                    println!("【设置壁纸】Windows 设置失败: {}", e);
                    return Err(format!("Windows 设置失败: {}", e));
                }
            }
        }
        "macos" => {
            let cmd = format!(
                "tell application \"System Events\" to set picture of every desktop to \"{}\"",
                image_path
            );
            println!("【设置壁纸】macOS 平台，执行 osascript 命令: {}", cmd);
            let result = Command::new("osascript")
                .args(&["-e", &cmd])
                .spawn();
            match result {
                Ok(_) => println!("【设置壁纸】macOS 壁纸设置命令已执行"),
                Err(e) => {
                    println!("【设置壁纸】macOS 设置失败: {}", e);
                    return Err(format!("macOS 设置失败: {}", e));
                }
            }
        }
        "linux" => {
            if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
                println!("【设置壁纸】Linux 平台，桌面环境: {}", desktop);
                if desktop.contains("GNOME") {
                    let cmd = format!("file://{}", image_path);
                    println!("【设置壁纸】GNOME，执行 gsettings 命令，图片 URI: {}", cmd);
                    let result = Command::new("gsettings")
                        .args(&[
                            "set",
                            "org.gnome.desktop.background",
                            "picture-uri",
                            &cmd,
                        ])
                        .spawn();
                    match result {
                        Ok(_) => println!("【设置壁纸】GNOME 壁纸设置命令已执行"),
                        Err(e) => {
                            println!("【设置壁纸】GNOME 设置失败: {}", e);
                            return Err(format!("GNOME 设置失败: {}", e));
                        }
                    }
                } else if desktop.contains("KDE") {
                    let script = format!(
                        "string:var allDesktops = desktops(); \
                        for (i=0; i<allDesktops.length; i++) {{ \
                        d = allDesktops[i]; \
                        d.wallpaperPlugin = \"org.kde.image\"; \
                        d.currentConfigGroup = Array(\"Wallpaper\", \"org.kde.image\", \"General\"); \
                        d.writeConfig(\"Image\", \"file://{}\") }}",
                        image_path
                    );
                    println!("【设置壁纸】KDE，执行 dbus-send 命令，脚本: {}", script);
                    let result = Command::new("dbus-send")
                        .args(&[
                            "--session",
                            "--dest=org.kde.plasmashell",
                            "--type=method_call",
                            "/PlasmaShell",
                            "org.kde.PlasmaShell.evaluateScript",
                            &script,
                        ])
                        .spawn();
                    match result {
                        Ok(_) => println!("【设置壁纸】KDE 壁纸设置命令已执行"),
                        Err(e) => {
                            println!("【设置壁纸】KDE 设置失败: {}", e);
                            return Err(format!("KDE 设置失败: {}", e));
                        }
                    }
                } else {
                    println!("【设置壁纸】不支持的 Linux 桌面环境: {}", desktop);
                    return Err("不支持的 Linux 桌面环境".into());
                }
            } else {
                println!("【设置壁纸】无法检测 Linux 桌面环境");
                return Err("无法检测 Linux 桌面环境".into());
            }
        }
        _ => {
            println!("【设置壁纸】不支持的平台: {}", platform);
            return Err("不支持的平台".into());
        }
    }
    println!("【设置壁纸】壁纸设置流程结束");
    Ok(())
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

/// 根据主显示器分辨率比例裁剪图片，并添加 _wall 后缀保存
/// 参数 image_path 必须为绝对路径
#[tauri::command]
async fn crop_image_to_screen_ratio(
    app_handle: AppHandle,
    image_path: String,
) -> Result<String, String> {
    println!("【裁剪壁纸】收到图片路径: {}", image_path);

    // 获取主显示器分辨率
    let monitor = app_handle.primary_monitor()
        .map_err(|e| e.to_string())?
        .ok_or("未找到主显示器".to_string())?;
    let screen_width = monitor.size().width as f64;
    let screen_height = monitor.size().height as f64;
    let screen_ratio = screen_width / screen_height;
    println!("【裁剪壁纸】主显示器分辨率: {}x{}, 屏幕比例: {}", screen_width, screen_height, screen_ratio);

    // 打开图片
    let path = Path::new(&image_path);
    let mut img = image::open(path)
        .map_err(|e| format!("打开图片失败: {}", e))?;
    let (img_width, img_height) = img.dimensions();
    // black_border 基于原始宽度
    let black_border = (img_width as f32 / 12.0).round() as u32;
    let black_border = black_border * 2;


    // 预处理图片, 先剪掉左边或者右边的黑边
    let now_utc = Utc::now();
    let japan_time = now_utc + Duration::hours(9);
    let japan_hour = japan_time.hour();
    
    if japan_hour < 6 {
       // 剪左边黑边，补到右边
       if img_width > black_border {
        // 1. 剪左边
        let cropped = img.crop(black_border, 0, img_width - black_border, img_width);
        // 2. 新建全黑画布
        let mut canvas = RgbaImage::from_pixel(img_width, img_height, Rgba([0, 0, 0, 255]));
        // 3. 把裁剪后的图片粘贴到左侧
        imageops::replace(&mut canvas, &cropped, 0, 0);
        img = DynamicImage::ImageRgba8(canvas);
        println!("【裁剪壁纸】剪左边黑边，补到右边，宽度: {}", black_border);
       }
    } else if japan_hour >= 18 {
        // 剪右边黑边，补到左边
        if img_width > black_border {
            // 1. 剪右边
            let cropped = img.crop(0, 0, img_width - black_border, img_width);
            // 2. 新建全黑画布
            let mut canvas = RgbaImage::from_pixel(img_width, img_height, Rgba([0, 0, 0, 255]));
            // 3. 把裁剪后的图片粘贴到右侧
            imageops::replace(&mut canvas, &cropped, black_border as i64, 0);
            img = DynamicImage::ImageRgba8(canvas);
            println!("【裁剪壁纸】剪右边黑边，补到左边，宽度: {}", black_border);
        }
        
    } else {
        println!("【裁剪壁纸】日本时间{}点，无需剪黑边", japan_hour);
    }
    // 剪黑边后，重新获取宽高和比例
    let (img_width, img_height) = img.dimensions();
    let img_ratio = img_width as f64 / img_height as f64;
    println!("【裁剪壁纸】剪黑边后图片尺寸: {}x{}, 新比例: {}", img_width, img_height, img_ratio);

    // 只做裁剪，不加黑边
    let cropped_img = if screen_ratio > img_ratio {
        // 屏幕更宽：以图片宽为基准，剪裁高度，保留上部
        let target_height = (img_width as f64 / screen_ratio).round() as u32;
        let crop_height = target_height.min(img_height);
        img.crop(0, 0, img_width, crop_height)
    } else {
        // 屏幕更窄：以图片高为基准，剪裁宽度，保留左部
        let target_width = (img_height as f64 * screen_ratio).round() as u32;
        let crop_width = target_width.min(img_width);
        img.crop(0, 0, crop_width, img_height)
    };

    // 保存裁剪后的图片
    let new_path = add_suffix_to_filename(path, "_wall");
    cropped_img.save(&new_path)
        .map_err(|e| format!("保存裁剪后图片失败: {}", e))?;
    println!("【裁剪壁纸】裁剪后图片已保存: {}", new_path.to_string_lossy());

    Ok(new_path.to_string_lossy().into_owned())
}

/// 辅助函数：为文件名添加后缀
fn add_suffix_to_filename<P: AsRef<Path>>(path: P, suffix: &str) -> PathBuf {
    let path = path.as_ref();
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = path.extension().map(|ext| format!(".{}", ext.to_string_lossy())).unwrap_or_default();
    let new_filename = format!("{}{}{}", stem, suffix, extension);
    parent.join(new_filename)
}

/// 先按屏幕比例裁剪图片，再设置为壁纸
#[tauri::command]
async fn crop_and_set_wallpaper(
    app_handle: AppHandle,
    image_path: String,
    platform: String,
) -> Result<String, String> {
    // 第一步：裁剪图片
    let cropped_path = crop_image_to_screen_ratio(app_handle.clone(), image_path.clone()).await
        .map_err(|e| format!("图片裁剪失败: {}", e))?;

    // 第二步：设置壁纸
    set_wallpaper(cropped_path.clone(), platform)
        .await
        .map_err(|e| format!("设置壁纸失败: {}", e))?;

    // 返回裁剪后图片路径
    Ok(cropped_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
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
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            update_earth_image,
            set_wallpaper,
            get_image_dir,
            crop_image_to_screen_ratio,
            crop_and_set_wallpaper
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
