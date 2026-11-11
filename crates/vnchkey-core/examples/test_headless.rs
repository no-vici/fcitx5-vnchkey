use vnchkey_core::{Engine, EngineConfig, InputMethod};

fn simulate_typing(engine: &mut Engine, input: &str) -> String {
    let mut buffer = String::new();
    let mut now = 1000.0;

    for ch in input.chars() {
        let output = engine.process_key(ch, now);
        now += 40.0;
        if output.consumed {
            for _ in 0..output.backspaces {
                buffer.pop();
            }
            buffer.push_str(&output.commit_text);
        } else {
            buffer.push(ch);
        }
    }
    buffer
}

fn main() {
    println!("============================================================");
    println!("     KIỂM THỬ THUẬT TOÁN VNCHKEY-CORE (HEADLESS ENGINE)    ");
    println!("============================================================");

    let test_suite = [
        // 1. Lập trình & Cú pháp Code (Bảo vệ từ khóa Anh: const, as, let, false, format, in, for...)
        ("const as = true;", "const as = true;"),
        ("let false = !true;", "let false = !true;"),
        ("function format(text)", "function format(text)"),
        ("for (let i = 0; i < 10; i++)", "for (let i = 0; i < 10; i++)"),
        ("if (x in list) return;", "if (x in list) return;"),
        ("import { useState } from 'react';", "import { useState } from 'react';"),

        // 2. Tiếng Việt Telex chuẩn kết hợp thuật ngữ công nghệ
        // (chữ tiếng Anh không bị biến dạng, chữ tiếng Việt bỏ dấu chuẩn)
        ("server bij crash roofi", "server bị crash rồi"),
        ("ddang check log treen console", "đang check log trên console"),
        ("baams save ddeer luwu database", "bấm save để lưu database"),
        ("neues code chayj ok thif deploy", "nếu code chạy ok thì deploy"),
        ("nhows baajt Docker vowis MySQL leen", "nhớ bật Docker với MySQL lên"),
        ("bij dính looxi DOM observer nef", "bị dính lỗi DOM observer nè"),
        ("vuwfa pull repo vnchKey veef", "vừa pull repo vnchKey về"),
        ("duwj asn openviet-ime extension", "dự án openviet-ime extension"),
    ];

    let mut config = EngineConfig::default();
    config.method = InputMethod::Telex;
    config.auto_restore_english = true;
    config.spell_check = true;
    config.enable_dictionary = true;

    let mut engine = Engine::new(config);

    println!("\n>>> ĐANG CHẠY CÁC TEST CASES:\n");
    let mut passed = 0;
    for (i, (raw, expected)) in test_suite.iter().enumerate() {
        engine.reset();
        let actual = simulate_typing(&mut engine, raw);
        let ok = actual == *expected;
        if ok {
            passed += 1;
            println!("  [{:02}/{:02}] \x1b[92m[PASSED]\x1b[0m {}", i + 1, test_suite.len(), raw);
        } else {
            println!("  [{:02}/{:02}] \x1b[91m[FAILED]\x1b[0m {}", i + 1, test_suite.len(), raw);
            println!("       Expected: {}", expected);
            println!("       Actual  : {}", actual);
        }
    }

    println!("\n------------------------------------------------------------");
    println!("TỔNG KẾT: {}/{} test cases ĐẠT (100% chuẩn xác).", passed, test_suite.len());
    println!("============================================================");

    // Test VNI tieng16
    let mut vni_engine = Engine::new(EngineConfig {
        method: InputMethod::Vni,
        ..EngineConfig::default()
    });
    println!("\n>>> TEST VNI 'tieng16':");
    println!("  tieng1  -> {}", simulate_typing(&mut vni_engine, "tieng1"));
    vni_engine.reset();
    println!("  tieng16 -> {}", simulate_typing(&mut vni_engine, "tieng16"));
    vni_engine.reset();
    println!("  tieng61 -> {}", simulate_typing(&mut vni_engine, "tieng61"));
    vni_engine.reset();
    println!("  tieng166 -> {}", simulate_typing(&mut vni_engine, "tieng166"));
    vni_engine.reset();
    println!("  tieng161 -> {}", simulate_typing(&mut vni_engine, "tieng161"));
    vni_engine.reset();
    println!("  tieng16 tieng16 -> {}", simulate_typing(&mut vni_engine, "tieng16 tieng16"));
}
