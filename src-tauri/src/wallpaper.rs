use chrono::{Duration, Timelike, Utc};
use image::{self, DynamicImage, GenericImageView, Rgba, RgbaImage, imageops};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use std::process::Command;

/// 为每个显示器创建独立的壁纸目录
pub fn create_monitor_wallpaper_dir(
    app: &AppHandle,
    monitor_index: usize,
) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("无法获取应用本地数据目录: {}", e))?;

    let mut base_path = PathBuf::from(app_data_dir);
    base_path.push("immediate_earth");
    base_path.push(format!("monitor_{}", monitor_index));

    fs::create_dir_all(&base_path).map_err(|e| format!("创建显示器壁纸目录失败: {}", e))?;

    Ok(base_path)
}

/// 为特定显示器裁剪图片，保持原有的裁剪逻辑
pub async fn crop_image_for_monitor(
    app: &AppHandle,
    image_path: &str,
    monitor_index: usize,
) -> Result<String, String> {
    let monitors = app
        .available_monitors()
        .map_err(|e| format!("获取显示器信息失败: {}", e))?;

    if monitor_index >= monitors.len() {
        return Err(format!("无效的显示器索引: {}", monitor_index));
    }

    let monitor = &monitors[monitor_index];
    let width = monitor.size().width;
    let height = monitor.size().height;
    let scale_factor = monitor.scale_factor();
    let target_width = (width as f64 * scale_factor) as u32;
    let target_height = (height as f64 * scale_factor) as u32;
    let screen_ratio = target_width as f64 / target_height as f64;

    let monitor_dir = create_monitor_wallpaper_dir(app, monitor_index)?;

    let path = Path::new(image_path);
    let mut img = image::open(path).map_err(|e| format!("打开图片失败: {}", e))?;

    let (img_width, img_height) = img.dimensions();
    let black_border = (img_width as f32 / 12.0).round() as u32 * 2;

    let screen_is_landscape = target_width >= target_height;
    let now_utc = Utc::now();
    let japan_time = now_utc + Duration::hours(9);
    let japan_hour = japan_time.hour();

    // 横屏处理黑边，竖屏不处理黑边
    if screen_is_landscape {
        // 处理黑边
        if japan_hour < 6 {
            // 凌晨
            if img_width > black_border {
                let cropped = img.crop(black_border, 0, img_width - black_border, img_width);
                let mut canvas = RgbaImage::from_pixel(img_width, img_height, Rgba([0, 0, 0, 255]));
                imageops::replace(&mut canvas, &cropped, 0, 0);
                img = DynamicImage::ImageRgba8(canvas);
            }
        } else if japan_hour >= 15 {
            // 下午
            if img_width > black_border {
                let cropped = img.crop(0, 0, img_width - black_border, img_width);
                let mut canvas = RgbaImage::from_pixel(img_width, img_height, Rgba([0, 0, 0, 255]));
                imageops::replace(&mut canvas, &cropped, black_border as i64, 0);
                img = DynamicImage::ImageRgba8(canvas);
            }
        }
    }
    // 重新获取处理后的图片尺寸
    let (img_width, img_height) = img.dimensions();
    let img_ratio = img_width as f64 / img_height as f64;
    let cropped_img = if screen_ratio > img_ratio {
        // 如果屏幕比例大于图片比例，则裁剪高度
        let target_height = (img_width as f64 / screen_ratio).round() as u32;
        let crop_height = target_height.min(img_height);
        if screen_is_landscape || japan_hour >= 6 {
            // 横屏或竖屏非凌晨，裁剪顶部
            img.crop(0, 0, img_width, crop_height)
        } else {
            // 凌晨，保留底部
            img.crop(0, img_height - crop_height, img_width, crop_height)
        }
    } else {
        // 如果屏幕比例小于图片比例，则裁剪宽度
        let target_width = (img_height as f64 * screen_ratio).round() as u32;
        let crop_width = target_width.min(img_width);
        if screen_is_landscape || japan_hour >= 6 {
            // 横屏或竖屏非凌晨，裁剪左侧
            img.crop(0, 0, crop_width, img_height)
        } else {
            // 凌晨，保留右侧
            img.crop(img_width - crop_width, 0, crop_width, img_height)
        }
    };
    let new_path = monitor_dir.join(format!("wallpaper_{}x{}.png", width, height));
    cropped_img
        .save(&new_path)
        .map_err(|e| format!("保存裁剪后图片失败: {}", e))?;
    Ok(new_path.to_string_lossy().into_owned())
}

/// 为单个显示器设置壁纸
pub async fn set_wallpaper_for_monitor(
    image_path: String,
    platform: String,
    monitor_index: usize,
) -> Result<(), String> {
    match platform.as_str() {
        "windows" => {
            #[cfg(target_os = "windows")]{
            use windows::Win32::{UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SPIF_SENDCHANGE}};
            use windows::core::{PCWSTR};

            // 将路径转换为宽字符串
            let wide_path: Vec<u16> = image_path.encode_utf16().chain(std::iter::once(0)).collect();
            let pcwstr_path = PCWSTR(wide_path.as_ptr());

            // 验证文件路径是否存在
            if !Path::new(&image_path).exists() {
                return Err(format!("壁纸文件不存在: {}
请检查文件路径是否正确", image_path));
            }

            // 设置壁纸
            let result = unsafe {
                SystemParametersInfoW(
                    SPI_SETDESKWALLPAPER,
                    0,
                    Some(pcwstr_path.0 as *mut _), 
                    SPIF_UPDATEINIFILE | SPIF_SENDCHANGE
                )
            };

            if result.is_err() {
                 return Err(format!("SystemParametersInfoW调用失败
 文件路径: {}", image_path));
             };
            }
        }
        "macos" => {
            #[cfg(target_os = "macos")] {
            let cmd = format!(
                "tell application \"System Events\" to set picture of desktop {} to \"{}\"",
                monitor_index + 1,
                image_path
            );
            Command::new("osascript")
                .args(&["-e", &cmd])
                .spawn()
                .map_err(|e| format!("macOS 设置失败: {}", e))?;
            }
        }
        "linux" => {
            #[cfg(target_os = "linux")] {
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
        }
        _ => return Err("不支持的平台".into()),
    }
    Ok(())
}
