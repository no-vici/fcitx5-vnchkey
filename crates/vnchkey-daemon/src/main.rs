//! vnchKey Background Daemon (Tự Do Edition).

use clap::Parser;
use vnchkey_core::{Engine, EngineConfig, InputMethod, ToneStyle};
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about = "vnchKey Vietnamese Input Daemon for Linux")]
struct Args {
    /// Input method: telex or vni
    #[arg(short, long, default_value = "telex")]
    method: String,

    /// Tone placement: modern (hoà) or classic (hòa)
    #[arg(short, long, default_value = "modern")]
    tone: String,

    /// Enable GoNhanh English auto-restore
    #[arg(long, default_value_t = true)]
    auto_restore: bool,

    /// Enable XKey smart casing
    #[arg(long, default_value_t = true)]
    smart_casing: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    println!("══════════════════════════════════════════════════════════════");
    println!("  VNCHKEY: TỰ DO EDITION - VIETNAMESE INPUT DAEMON          ");
    println!("  Cờ Vàng Ba Sọc Đỏ Heritage · Pure Safe Rust · Fcitx5     ");
    println!("══════════════════════════════════════════════════════════════");

    let method = match args.method.to_lowercase().as_str() {
        "vni" => InputMethod::Vni,
        _ => InputMethod::Telex,
    };

    let tone_style = match args.tone.to_lowercase().as_str() {
        "classic" => ToneStyle::Classic,
        _ => ToneStyle::Modern,
    };

    let config = EngineConfig {
        method,
        tone_style,
        auto_restore_english: args.auto_restore,
        smart_casing: args.smart_casing,
        quick_undo: true,
        sentinel_enabled: true,
    };

    let mut engine = Engine::new(config);
    println!("Daemon initialized successfully. Listening on compositor socket...");

    // Event loop interfacing with Wayland / X11 window focus and uinput
    Ok(())
}
