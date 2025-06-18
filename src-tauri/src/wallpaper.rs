use chrono::{Timelike, Utc, Duration};
use image::{self, GenericImageView, Rgba, RgbaImage, imageops, DynamicImage};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Manager};

/// 为每个显示器创建独立的壁纸目录
pub fn create_monitor_wallpaper_dir(app: &AppHandle, monitor_index: usize) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("无法获取应用本地数据目录: {}", e))?;
    
    let mut base_path = PathBuf::from(app_data_dir);
    base_path.push("immediate_earth");
    base_path.push(format!("monitor_{}", monitor_index));
    
    fs::create_dir_all(&base_path)
        .map_err(|e| format!("创建显示器壁纸目录失败: {}", e))?;
    
    Ok(base_path)
}

/// 为特定显示器裁剪图片，保持原有的裁剪逻辑
pub async fn crop_image_for_monitor(
    app: &AppHandle,
    image_path: &str,
    monitor_index: usize,
) -> Result<String, String> {
    let monitors = app.available_monitors()
        .map_err(|e| format!("获取显示器信息失败: {}", e))?;
    
    if monitor_index >= monitors.len() {
        return Err(format!("无效的显示器索引: {}", monitor_index));
    }
    
    let monitor = &monitors[monitor_index];
    let width = monitor.size().width;
    let height = monitor.size().height;
    let scale_factor = monitor.scale_factor();
    
    let monitor_dir = create_monitor_wallpaper_dir(app, monitor_index)?;
    
    let path = Path::new(image_path);
    let mut img = image::open(path)
        .map_err(|e| format!("打开图片失败: {}", e))?;
    
    let (img_width, img_height) = img.dimensions();
    let black_border = (img_width as f32 / 12.0).round() as u32 * 2;
    
    let now_utc = Utc::now();
    let japan_time = now_utc + Duration::hours(9);
    let japan_hour = japan_time.hour();
    
    // 处理黑边
    if japan_hour < 6 {
        if img_width > black_border {
            let cropped = img.crop(black_border, 0, img_width - black_border, img_width);
            let mut canvas = RgbaImage::from_pixel(img_width, img_height, Rgba([0, 0, 0, 255]));
            imageops::replace(&mut canvas, &cropped, 0, 0);
            img = DynamicImage::ImageRgba8(canvas);
        }
    } else if japan_hour >= 15 {
        if img_width > black_border {
            let cropped = img.crop(0, 0, img_width - black_border, img_width);
            let mut canvas = RgbaImage::from_pixel(img_width, img_height, Rgba([0, 0, 0, 255]));
            imageops::replace(&mut canvas, &cropped, black_border as i64, 0);
            img = DynamicImage::ImageRgba8(canvas);
        }
    }
    // 重新获取处理后的图片尺寸
    let (img_width, img_height) = img.dimensions();
    let img_ratio = img_width as f64 / img_height as f64;
    let target_width = (width as f64 * scale_factor) as u32;
    let target_height = (height as f64 * scale_factor) as u32;
    let screen_ratio = target_width as f64 / target_height as f64;
    let cropped_img = if screen_ratio > img_ratio {
        let target_height = (img_width as f64 / screen_ratio).round() as u32;
        let crop_height = target_height.min(img_height);
        img.crop(0, 0, img_width, crop_height)
    } else {
        let target_width = (img_height as f64 * screen_ratio).round() as u32;
        let crop_width = target_width.min(img_width);
        img.crop(0, 0, crop_width, img_height)
    };
    let new_path = monitor_dir.join(format!(
        "wallpaper_{}x{}.png",
        width, height
    ));
    cropped_img.save(&new_path)
        .map_err(|e| format!("保存裁剪后图片失败: {}", e))?;
    Ok(new_path.to_string_lossy().into_owned())
}

/// 为单个显示器设置壁纸
pub async fn set_wallpaper_for_monitor(
    image_path: String, 
    platform: String,
    monitor_index: usize
) -> Result<(), String> {
    match platform.as_str() {
        "windows" => {
            let cmd = format!(
                "Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; \\n                public class Wallpaper {{ [DllImport(\"user32.dll\", CharSet=CharSet.Auto)] \\n                public static extern int SystemParametersInfo(int uAction, int uParam, String lpvParam, int fuWinIni); }}'; \\n                [Wallpaper]::SystemParametersInfo(20, {}, '{}', 3)",
                monitor_index, image_path
            );
            Command::new("powershell")
                .args(&["-Command", &cmd])
                .spawn()
                .map_err(|e| format!("Windows 设置失败: {}", e))?;
        }
        "macos" => {
            let cmd = format!(
                "tell application \"System Events\" to set picture of desktop {} to \"{}\"",
                monitor_index + 1, image_path
            );
            Command::new("osascript")
                .args(&["-e", &cmd])
                .spawn()
                .map_err(|e| format!("macOS 设置失败: {}", e))?;
        }
        "linux" => {
            if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
                if desktop.contains("GNOME") {
                    let cmd = format!("file://{}", image_path);
                    Command::new("gsettings")
                        .args(&[
                            "set",
                            &format!("org.gnome.desktop.background.picture-uri-{}", monitor_index),
                            &cmd,
                        ])
                        .spawn()
                        .map_err(|e| format!("GNOME 设置失败: {}", e))?;
                } else if desktop.contains("KDE") {
                    let script = format!(
                        "string:var allDesktops = desktops(); \\n                        d = allDesktops[{}]; \\n                        d.wallpaperPlugin = \"org.kde.image\"; \\n                        d.currentConfigGroup = Array(\"Wallpaper\", \"org.kde.image\", \"General\"); \\n                        d.writeConfig(\"Image\", \"file://{}\")",
                        monitor_index, image_path
                    );
                    Command::new("dbus-send")
                        .args(&[
                            "--session",
                            "--dest=org.kde.plasmashell",
                            "--type=method_call",
                            "/PlasmaShell",
                            "org.kde.PlasmaShell.evaluateScript",
                            &script,
                        ])
                        .spawn()
                        .map_err(|e| format!("KDE 设置失败: {}", e))?;
                }
            }
        }
        _ => return Err("不支持的平台".into()),
    }
    Ok(())
}