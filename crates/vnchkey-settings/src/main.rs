mod app;
mod config;
pub mod doctor;
pub mod system_apps;

use app::SettingsApp;
use eframe::egui;

fn load_icon() -> Option<egui::IconData> {
    let png_bytes = include_bytes!("../../../assets/logo.png");
    if let Ok(img) = image::load_from_memory(png_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--doctor" || a == "-d" || a == "doctor") {
        let report = doctor::DoctorReport::run();
        report.print_cli();
        return Ok(());
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Fcitx5 vnchKey Settings & Diagnostic Tool");
        println!("Cách dùng:");
        println!("  fcitx5-vnchkey-settings           Mở giao diện cài đặt đồ họa (GUI)");
        println!("  fcitx5-vnchkey-settings --doctor  Chẩn đoán môi trường Fcitx5 & tiến trình Electron (/proc/PID)");
        return Ok(());
    }
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Fcitx5 vnchKey Settings")
        .with_inner_size([840.0, 580.0])
        .with_min_inner_size([780.0, 520.0]);

    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Fcitx5 vnchKey Settings",
        native_options,
        Box::new(|cc| Ok(Box::new(SettingsApp::new(cc)))),
    )
}
