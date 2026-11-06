//! Eframe Application GUI implementation for Fcitx5 vnchKey Settings.

use crate::config::{ConfigManager, VnchkeyConfig};
use eframe::egui;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabPage {
    General,
    Typing,
    Applications,
    Macros,
    Dictionary,
    Keymap,
    Shortcuts,
    Appearance,
    Backup,
    About,
}

pub struct SettingsApp {
    current_page: TabPage,
    config: VnchkeyConfig,
    macros: Vec<(String, String)>,
    dictionary: Vec<String>,
    app_modes: Vec<(String, String)>,
    status_message: Option<(String, bool)>, // (message, is_success)

    // Form editing states
    new_dict_word: String,
    dict_search: String,

    // Logo texture
    logo_texture: Option<egui::TextureHandle>,
}

impl SettingsApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Setup Catppuccin Mocha / Lilypad dark theme styling
        let mut visuals = egui::Visuals::dark();
        visuals.override_text_color = Some(egui::Color32::from_rgb(0xcd, 0xd6, 0xf4));
        visuals.panel_fill = egui::Color32::from_rgb(0x1e, 0x1e, 0x2d);
        visuals.window_fill = egui::Color32::from_rgb(0x1e, 0x1e, 0x2d);
        visuals.faint_bg_color = egui::Color32::from_rgb(0x18, 0x18, 0x25);
        visuals.extreme_bg_color = egui::Color32::from_rgb(0x18, 0x18, 0x25);

        // Widgets styling
        visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(0x1e, 0x1e, 0x2d);
        visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0xcd, 0xd6, 0xf4));

        visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(0x25, 0x28, 0x39);
        visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x3b, 0x3e, 0x52));
        visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);

        visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x38, 0x3a, 0x54);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x58, 0x56, 0xd6));
        visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);

        visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0x58, 0x56, 0xd6);
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x58, 0x56, 0xd6));
        visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);

        cc.egui_ctx.set_visuals(visuals);

        // Setup Vietnamese font with complete diacritics support (Roboto Regular)
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "roboto_vietnamese".to_owned(),
            std::sync::Arc::new(egui::FontData::from_static(include_bytes!(
                "../../../assets/fonts/Roboto-Regular.ttf"
            ))),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "roboto_vietnamese".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("roboto_vietnamese".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        // Load logo texture
        let logo_texture = Self::load_logo_texture(&cc.egui_ctx);

        Self {
            current_page: TabPage::General,
            config: ConfigManager::load_config(),
            macros: ConfigManager::load_macros(),
            dictionary: ConfigManager::load_dictionary(),
            app_modes: ConfigManager::load_app_modes(),
            status_message: None,
            new_dict_word: String::new(),
            dict_search: String::new(),
            logo_texture,
        }
    }

    fn load_logo_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
        let png_bytes = include_bytes!("../../../assets/logo.png");
        if let Ok(img) = image::load_from_memory(png_bytes) {
            let rgba = img.to_rgba8();
            let size = [rgba.width() as usize, rgba.height() as usize];
            let pixels = rgba.into_raw();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            Some(ctx.load_texture("vnchkey-logo", color_image, egui::TextureOptions::LINEAR))
        } else {
            None
        }
    }

    fn save_all(&mut self) -> bool {
        let ok1 = ConfigManager::save_config(&self.config).is_ok();
        let ok2 = ConfigManager::save_macros(&self.macros).is_ok();
        let ok3 = ConfigManager::save_dictionary(&self.dictionary).is_ok();
        let ok4 = ConfigManager::save_app_modes(&self.app_modes).is_ok();

        if ok1 && ok2 && ok3 && ok4 {
            self.status_message = Some(("Cài đặt đã được lưu và cập nhật Fcitx5!".to_string(), true));
            true
        } else {
            self.status_message = Some(("Đã xảy ra lỗi khi lưu cấu hình!".to_string(), false));
            false
        }
    }
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Bottom Control Bar
        egui::TopBottomPanel::bottom("bottom_bar")
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(0x18, 0x18, 0x25))
                    .inner_margin(egui::Margin::symmetric(20, 12))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x28, 0x2a, 0x36))),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("⟲ Khôi Phục Mặc Định").clicked() {
                        ConfigManager::reset_to_defaults();
                        self.config = ConfigManager::load_config();
                        self.macros = ConfigManager::load_macros();
                        self.dictionary = ConfigManager::load_dictionary();
                        self.app_modes = ConfigManager::load_app_modes();
                        self.status_message = Some(("Đã khôi phục cài đặt gốc mặc định!".to_string(), true));
                    }

                    if let Some((msg, is_success)) = &self.status_message {
                        ui.add_space(12.0);
                        let color = if *is_success {
                            egui::Color32::from_rgb(0xa6, 0xe3, 0xa1)
                        } else {
                            egui::Color32::from_rgb(0xf3, 0x8b, 0xa8)
                        };
                        let (icon_rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                        let c = icon_rect.center();
                        let stroke = egui::Stroke::new(2.0_f32, color);
                        if *is_success {
                            ui.painter().line_segment([egui::pos2(c.x - 5.0, c.y + 0.5), egui::pos2(c.x - 1.5, c.y + 4.0)], stroke);
                            ui.painter().line_segment([egui::pos2(c.x - 1.5, c.y + 4.0), egui::pos2(c.x + 5.5, c.y - 3.5)], stroke);
                        } else {
                            ui.painter().line_segment([egui::pos2(c.x - 4.0, c.y - 4.0), egui::pos2(c.x + 4.0, c.y + 4.0)], stroke);
                            ui.painter().line_segment([egui::pos2(c.x + 4.0, c.y - 4.0), egui::pos2(c.x - 4.0, c.y + 4.0)], stroke);
                        }
                        ui.colored_label(color, msg);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Primary OK Button with custom vector checkmark (no tofu box)
                        let (ok_rect, ok_resp) = ui.allocate_exact_size(egui::vec2(118.0, 28.0), egui::Sense::click());
                        let ok_bg = if ok_resp.is_pointer_button_down_on() {
                            egui::Color32::from_rgb(0x48, 0x46, 0xc0)
                        } else if ok_resp.hovered() {
                            egui::Color32::from_rgb(0x68, 0x66, 0xe6)
                        } else {
                            egui::Color32::from_rgb(0x58, 0x56, 0xd6)
                        };
                        ui.painter().rect_filled(ok_rect, egui::CornerRadius::same(6), ok_bg);

                        // Vector checkmark on left of OK button
                        let ck_x = ok_rect.left() + 16.0;
                        let ck_y = ok_rect.center().y;
                        let ck_stroke = egui::Stroke::new(1.8_f32, egui::Color32::WHITE);
                        ui.painter().line_segment(
                            [egui::pos2(ck_x - 4.0, ck_y + 0.5), egui::pos2(ck_x - 1.0, ck_y + 3.5)],
                            ck_stroke,
                        );
                        ui.painter().line_segment(
                            [egui::pos2(ck_x - 1.0, ck_y + 3.5), egui::pos2(ck_x + 5.0, ck_y - 3.0)],
                            ck_stroke,
                        );
                        ui.painter().text(
                            egui::pos2(ok_rect.left() + 28.0, ok_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            "Đồng Ý (OK)",
                            egui::FontId::proportional(13.0),
                            egui::Color32::WHITE,
                        );

                        if ok_resp.clicked() {
                            if self.save_all() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }

                        // Apply Button
                        if ui.add(egui::Button::new("Áp Dụng").min_size(egui::vec2(80.0, 28.0))).clicked() {
                            self.save_all();
                        }

                        // Cancel Button
                        if ui.add(egui::Button::new("Huỷ").min_size(egui::vec2(80.0, 28.0))).clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                });
            });

        // Left Navigation Sidebar
        egui::SidePanel::left("sidebar_panel")
            .exact_width(190.0)
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(0x18, 0x18, 0x25))
                    .inner_margin(egui::Margin::symmetric(10, 16))
                    .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x28, 0x2a, 0x36))),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 4.0);

                let nav_items = [
                    (TabPage::General, "General"),
                    (TabPage::Typing, "Typing"),
                    (TabPage::Applications, "Applications"),
                    (TabPage::Macros, "Macros"),
                    (TabPage::Dictionary, "Dictionary"),
                    (TabPage::Keymap, "Keymap"),
                    (TabPage::Shortcuts, "Shortcuts"),
                    (TabPage::Appearance, "Appearance"),
                    (TabPage::Backup, "Backup"),
                ];

                for (page, title) in nav_items {
                    self.render_nav_button(ui, page, title);
                }

                // 20px Spacer between main categories and About matching lilypad
                ui.add_space(20.0);

                self.render_nav_button(ui, TabPage::About, "About");
            });

        // Central Content Area
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(0x1e, 0x1e, 0x2d))
                    .inner_margin(egui::Margin::symmetric(36, 24)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        match self.current_page {
                            TabPage::General => self.show_general_page(ui),
                            TabPage::Typing => self.show_typing_page(ui),
                            TabPage::Applications => self.show_applications_page(ui),
                            TabPage::Macros => self.show_macros_page(ui),
                            TabPage::Dictionary => self.show_dictionary_page(ui),
                            TabPage::Keymap => self.show_keymap_page(ui),
                            TabPage::Shortcuts => self.show_shortcuts_page(ui),
                            TabPage::Appearance => self.show_appearance_page(ui),
                            TabPage::Backup => self.show_backup_page(ui),
                            TabPage::About => self.show_about_page(ui),
                        }
                    });
            });
    }
}

impl SettingsApp {
    fn render_nav_button(&mut self, ui: &mut egui::Ui, page: TabPage, title: &str) {
        let is_selected = self.current_page == page;
        let (rect, res) = ui.allocate_exact_size(egui::vec2(170.0, 36.0), egui::Sense::click());

        let bg_color = if is_selected {
            egui::Color32::from_rgb(0x4b, 0x44, 0x63)
        } else if res.hovered() {
            egui::Color32::from_rgb(0x28, 0x28, 0x3d)
        } else {
            egui::Color32::TRANSPARENT
        };

        if bg_color != egui::Color32::TRANSPARENT {
            ui.painter()
                .rect_filled(rect, egui::CornerRadius::same(8), bg_color);
        }

        let text_color = if is_selected {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_rgb(0xa6, 0xad, 0xc8)
        };

        // Paint crisp colorful vector icon matching fcitx5-lilypad sidebar icons
        let icon_center = egui::pos2(rect.left() + 20.0, rect.center().y);
        Self::paint_nav_icon(ui.painter(), page, icon_center);

        // Paint tab label
        ui.painter().text(
            egui::pos2(rect.left() + 38.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            title,
            egui::FontId::proportional(13.0),
            text_color,
        );

        if res.clicked() {
            self.current_page = page;
            self.status_message = None;
        }
    }

    fn paint_nav_icon(painter: &egui::Painter, page: TabPage, c: egui::Pos2) {
        let white = egui::Color32::from_rgb(0xec, 0xef, 0xf4);
        let dark = egui::Color32::from_rgb(0x18, 0x18, 0x25);
        let light_grey = egui::Color32::from_rgb(0xd8, 0xde, 0xe9);

        match page {
            TabPage::General => {
                // Green badge with white wrench (matching Lilypad General icon)
                let r = egui::Rect::from_center_size(c, egui::vec2(15.0, 15.0));
                painter.rect_filled(r, egui::CornerRadius::same(4), egui::Color32::from_rgb(0x26, 0xa2, 0x69));
                painter.line_segment(
                    [egui::pos2(c.x, c.y - 2.0), egui::pos2(c.x, c.y + 4.5)],
                    egui::Stroke::new(2.2_f32, white),
                );
                painter.circle_stroke(egui::pos2(c.x, c.y - 3.0), 2.5, egui::Stroke::new(1.8_f32, white));
                painter.rect_filled(
                    egui::Rect::from_center_size(egui::pos2(c.x, c.y - 4.8), egui::vec2(2.4, 2.4)),
                    egui::CornerRadius::ZERO,
                    egui::Color32::from_rgb(0x26, 0xa2, 0x69),
                );
            }
            TabPage::Typing => {
                // White/grey keyboard icon
                let r = egui::Rect::from_center_size(c, egui::vec2(16.0, 11.0));
                painter.rect_filled(r, egui::CornerRadius::same(2), light_grey);
                for dx in [-4.5_f32, -1.5, 1.5, 4.5] {
                    painter.rect_filled(
                        egui::Rect::from_center_size(egui::pos2(c.x + dx, c.y - 2.0), egui::vec2(1.8, 1.8)),
                        egui::CornerRadius::ZERO,
                        dark,
                    );
                }
                painter.rect_filled(
                    egui::Rect::from_center_size(egui::pos2(c.x, c.y + 2.2), egui::vec2(8.0, 1.8)),
                    egui::CornerRadius::ZERO,
                    dark,
                );
            }
            TabPage::Applications => {
                // Slate-purple circle with 3 cyan dots (matching Lilypad Applications icon)
                painter.circle_filled(c, 7.5, egui::Color32::from_rgb(0x45, 0x47, 0x5a));
                let dot_color = egui::Color32::from_rgb(0x74, 0xc7, 0xec);
                for dx in [-3.5_f32, 0.0, 3.5] {
                    painter.circle_filled(egui::pos2(c.x + dx, c.y), 1.3, dot_color);
                }
            }
            TabPage::Macros => {
                // Orange-red circle with white diagonal pencil (matching Lilypad Macros icon)
                painter.circle_filled(c, 7.5, egui::Color32::from_rgb(0xff, 0x5e, 0x3a));
                painter.line_segment(
                    [egui::pos2(c.x - 3.0, c.y + 3.0), egui::pos2(c.x + 3.0, c.y - 3.0)],
                    egui::Stroke::new(2.2_f32, white),
                );
                painter.circle_filled(egui::pos2(c.x - 3.5, c.y + 3.5), 1.0, egui::Color32::from_rgb(0xf9, 0xe2, 0xaf));
            }
            TabPage::Dictionary => {
                // Two overlapping pages (matching Lilypad Dictionary icon)
                let r_back = egui::Rect::from_center_size(egui::pos2(c.x - 2.0, c.y - 1.0), egui::vec2(9.0, 11.0));
                let r_front = egui::Rect::from_center_size(egui::pos2(c.x + 2.0, c.y + 1.5), egui::vec2(9.0, 11.0));
                painter.rect_stroke(r_back, egui::CornerRadius::same(1), egui::Stroke::new(1.5_f32, light_grey), egui::StrokeKind::Inside);
                painter.rect_filled(r_front, egui::CornerRadius::same(1), dark);
                painter.rect_stroke(r_front, egui::CornerRadius::same(1), egui::Stroke::new(1.5_f32, white), egui::StrokeKind::Inside);
            }
            TabPage::Keymap | TabPage::Shortcuts => {
                // Keyboard with warm top bar + grey body (matching Lilypad Keymap/Shortcuts icons)
                let top = egui::Rect::from_center_size(egui::pos2(c.x, c.y - 4.0), egui::vec2(16.0, 2.5));
                let body = egui::Rect::from_center_size(egui::pos2(c.x, c.y + 1.0), egui::vec2(16.0, 8.5));
                painter.rect_filled(top, egui::CornerRadius::same(1), egui::Color32::from_rgb(0xf9, 0xe2, 0xaf));
                painter.rect_filled(body, egui::CornerRadius::same(2), light_grey);
                for dx in [-4.5_f32, -1.5, 1.5, 4.5] {
                    painter.rect_filled(
                        egui::Rect::from_center_size(egui::pos2(c.x + dx, c.y - 0.5), egui::vec2(1.8, 1.8)),
                        egui::CornerRadius::ZERO,
                        dark,
                    );
                }
                painter.rect_filled(
                    egui::Rect::from_center_size(egui::pos2(c.x, c.y + 3.0), egui::vec2(8.0, 1.6)),
                    egui::CornerRadius::ZERO,
                    dark,
                );
            }
            TabPage::Appearance => {
                // Blue window/monitor badge with white inner area (matching Lilypad Appearance icon)
                let outer = egui::Rect::from_center_size(c, egui::vec2(16.0, 12.0));
                let inner = egui::Rect::from_center_size(egui::pos2(c.x - 1.5, c.y + 0.5), egui::vec2(9.0, 6.5));
                painter.rect_filled(outer, egui::CornerRadius::same(2), egui::Color32::from_rgb(0x1c, 0x71, 0xd8));
                painter.rect_filled(inner, egui::CornerRadius::same(1), white);
            }
            TabPage::Backup => {
                // Floppy disk icon (matching Lilypad Backup icon)
                let outer = egui::Rect::from_center_size(c, egui::vec2(14.0, 14.0));
                painter.rect_filled(outer, egui::CornerRadius::same(2), light_grey);
                let top_slot = egui::Rect::from_center_size(egui::pos2(c.x, c.y - 4.0), egui::vec2(8.0, 4.5));
                let bot_label = egui::Rect::from_center_size(egui::pos2(c.x, c.y + 3.5), egui::vec2(9.0, 5.5));
                painter.rect_filled(top_slot, egui::CornerRadius::same(1), dark);
                painter.rect_filled(bot_label, egui::CornerRadius::same(1), dark);
                painter.line_segment(
                    [egui::pos2(c.x, c.y + 1.5), egui::pos2(c.x, c.y + 5.5)],
                    egui::Stroke::new(1.5_f32, light_grey),
                );
            }
            TabPage::About => {
                // Solid white circle with dark 'i' inside (matching Lilypad About icon)
                painter.circle_filled(c, 7.5, white);
                painter.circle_filled(egui::pos2(c.x, c.y - 3.2), 1.2, dark);
                painter.line_segment(
                    [egui::pos2(c.x, c.y - 0.5), egui::pos2(c.x, c.y + 4.2)],
                    egui::Stroke::new(2.0_f32, dark),
                );
            }
        }
    }

    fn render_header(&self, ui: &mut egui::Ui, title: &str, subtitle: &str) {
        // Clean typography matching lilypad - regular font weight, no thick distracting font
        ui.label(
            egui::RichText::new(title)
                .size(22.0)
                .color(egui::Color32::from_rgb(0xcd, 0xd6, 0xf4)),
        );
        ui.label(
            egui::RichText::new(subtitle)
                .size(13.0)
                .color(egui::Color32::from_rgb(0x7f, 0x84, 0x9c)),
        );
        ui.add_space(18.0);
    }

    fn render_group_header(&self, ui: &mut egui::Ui, title: &str) {
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(title.to_uppercase())
                .size(12.0)
                .color(egui::Color32::from_rgb(0x89, 0xb4, 0xfa)),
        );
        ui.add_space(8.0);
    }

    // 1. General Page
    fn show_general_page(&mut self, ui: &mut egui::Ui) {
        self.render_header(ui, "General", "Configure basic input method settings and behaviors.");

        self.render_group_header(ui, "Input Method");

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Input Method").size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("im_method")
                    .selected_text(&self.config.input_method)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.input_method, "VNI".to_string(), "VNI");
                        ui.selectable_value(&mut self.config.input_method, "Telex".to_string(), "Telex");
                        ui.selectable_value(&mut self.config.input_method, "Auto".to_string(), "Auto (Telex & VNI)");
                    });
            });
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Output Charset").size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("output_charset")
                    .selected_text("Unicode (Standard)")
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.output_charset, "Unicode".to_string(), "Unicode (Standard)");
                    });
            });
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Chế độ xuất chữ (Output Mode)").size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let current_label = if self.config.output_mode == "Preedit" {
                    "Gạch Chân (Preedit - Web/Google Docs)"
                } else {
                    "Trực Tiếp (Direct - Terminal/Editor)"
                };
                egui::ComboBox::from_id_salt("output_mode")
                    .selected_text(current_label)
                    .width(260.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.output_mode, "Direct".to_string(), "Trực Tiếp (Direct - Terminal/Editor)");
                        ui.selectable_value(&mut self.config.output_mode, "Preedit".to_string(), "Gạch Chân (Preedit - Web/Google Docs)");
                    });
            });
        });
        ui.add_space(8.0);

        ui.checkbox(
            &mut self.config.remember_app_mode,
            "Tự động nhớ chế độ xuất chữ cho từng ứng dụng (Direct / Preedit)",
        );
        ui.add_space(8.0);

        ui.checkbox(
            &mut self.config.preedit_tooltip,
            "Khung chữ nổi khi gõ gạch chân (Floating Preedit Tooltip - khuyên dùng cho Google Docs)",
        );
        ui.add_space(8.0);

        ui.checkbox(
            &mut self.config.persistent_context_guard,
            "Persistent Context Guard: Chống mất chữ khi hiện popup gợi ý trong IntelliJ / PyCharm / Wine",
        );
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Độ trễ khi gõ Direct trên Web (Direct Delay)").size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(egui::Slider::new(&mut self.config.direct_delay_ms, 0..=20).suffix(" ms (0 = Tự động 4ms)"));
            });
        });

        self.render_group_header(ui, "Typing");

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Type w to Produce ư").size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("type_w2u")
                    .selected_text(&self.config.type_w_to_u)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.type_w_to_u, "Non-Start".to_string(), "Non-Start");
                        ui.selectable_value(&mut self.config.type_w_to_u, "Always".to_string(), "Always");
                        ui.selectable_value(&mut self.config.type_w_to_u, "Disabled".to_string(), "Disabled");
                    });
            });
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Type [ -> ơ, ] -> ư, { -> Ơ, } -> Ư").size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("bracket_transform")
                    .selected_text(&self.config.bracket_transform)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.bracket_transform, "Disabled".to_string(), "Disabled");
                        ui.selectable_value(&mut self.config.bracket_transform, "Enabled".to_string(), "Enabled");
                    });
            });
        });
    }

    // 2. Typing Page
    fn show_typing_page(&mut self, ui: &mut egui::Ui) {
        self.render_header(ui, "Typing", "Fine-tune spelling corrections and advanced typing options.");

        self.render_group_header(ui, "Spelling & Tone Options");

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Quy tắc đặt dấu thanh (Tone Style)").size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let current_label = if self.config.modern_style { "Modern: oà, uý, oè" } else { "Classic: hòa, ủy, hòe" };
                egui::ComboBox::from_id_salt("tone_style")
                    .selected_text(current_label)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.modern_style, true, "Modern: oà, uý, oè");
                        ui.selectable_value(&mut self.config.modern_style, false, "Classic: hòa, ủy, hòe");
                    });
            });
        });
        ui.add_space(14.0);

        ui.checkbox(
            &mut self.config.smart_horn,
            "Tự động thêm dấu râu cho 'uo' (duoc95 -> được, nuoc1 -> nước)",
        );
        ui.add_space(6.0);

        ui.checkbox(
            &mut self.config.spell_check,
            "Chánh Tả: Kiểm tra chánh tả âm tiết tiếng Việt",
        );
        ui.add_space(6.0);

        ui.checkbox(
            &mut self.config.auto_restore_english,
            "Restore English: Tự động khôi phục từ tiếng Anh (post, text, class...)",
        );
        ui.add_space(6.0);

        ui.checkbox(
            &mut self.config.smart_casing,
            "XKey: Giữ nguyên định dạng chữ hoa/chữ thường thông minh",
        );
        ui.add_space(6.0);

        ui.checkbox(
            &mut self.config.free_marking,
            "Cho phép bỏ dấu tự do trên bất kỳ nguyên âm nào",
        );
        ui.add_space(6.0);

        ui.checkbox(
            &mut self.config.fast_consonant_doubling,
            "Gõ nhanh phụ âm đầu (cc -> ch, gg -> gi, kk -> kh)",
        );
    }

    // 3. Applications Page
    fn show_applications_page(&mut self, ui: &mut egui::Ui) {
        self.render_header(ui, "Applications", "Configure per-application input behaviors and bypass exclusions.");

        egui::Frame::new()
            .fill(egui::Color32::from_rgb(0x18, 0x18, 0x25))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x28, 0x2a, 0x36)))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Ứng Dụng (App Name)").strong().color(egui::Color32::from_rgb(0x89, 0xb4, 0xfa)));
                    ui.add_space(180.0);
                    ui.label(egui::RichText::new("Chế Độ (Mode)").strong().color(egui::Color32::from_rgb(0x89, 0xb4, 0xfa)));
                });
                ui.separator();

                let mut to_remove = None;
                for (idx, (app, mode)) in self.app_modes.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(app).desired_width(220.0));
                        ui.add_space(10.0);
                        let display_text = match mode.as_str() {
                            "Preedit" => "Gạch Chân (Preedit)",
                            "Direct" => "Trực Tiếp (Direct)",
                            "Bypass (Raw English)" => "Bypass (Raw English)",
                            other => other,
                        };
                        egui::ComboBox::from_id_salt(format!("app_mode_{}", idx))
                            .selected_text(display_text)
                            .width(180.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(mode, "Direct".to_string(), "Trực Tiếp (Direct)");
                                ui.selectable_value(mode, "Preedit".to_string(), "Gạch Chân (Preedit)");
                                ui.selectable_value(mode, "Bypass (Raw English)".to_string(), "Bypass (Raw English)");
                            });
                        if ui.button("🗑").clicked() {
                            to_remove = Some(idx);
                        }
                    });
                }

                if let Some(idx) = to_remove {
                    self.app_modes.remove(idx);
                }
            });

        ui.add_space(10.0);
        if ui.button("+ Thêm Ứng Dụng").clicked() {
            self.app_modes.push(("".to_string(), "Bypass (Raw English)".to_string()));
        }
    }

    // 4. Macros Page
    fn show_macros_page(&mut self, ui: &mut egui::Ui) {
        self.render_header(ui, "Macros", "Define shortcuts that automatically expand into full phrases.");

        ui.horizontal(|ui| {
            ui.checkbox(&mut self.config.enable_macro, "Bật tính năng gõ tắt (Enable Macro)");
            ui.add_space(20.0);
            ui.checkbox(&mut self.config.capitalize_macro, "Viết hoa thông minh (Capitalize Macro)");
        });
        ui.add_space(12.0);

        egui::Frame::new()
            .fill(egui::Color32::from_rgb(0x18, 0x18, 0x25))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x28, 0x2a, 0x36)))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Từ Viết Tắt (Shortcut)").strong().color(egui::Color32::from_rgb(0x89, 0xb4, 0xfa)));
                    ui.add_space(120.0);
                    ui.label(egui::RichText::new("Nội Dung Thay Thế (Expansion)").strong().color(egui::Color32::from_rgb(0x89, 0xb4, 0xfa)));
                });
                ui.separator();

                let mut to_remove = None;
                for (idx, (k, v)) in self.macros.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(k).desired_width(160.0));
                        ui.add_space(10.0);
                        ui.add(egui::TextEdit::singleline(v).desired_width(260.0));
                        if ui.button("🗑").clicked() {
                            to_remove = Some(idx);
                        }
                    });
                }

                if let Some(idx) = to_remove {
                    self.macros.remove(idx);
                }
            });

        ui.add_space(10.0);
        if ui.button("+ Thêm Viết Tắt").clicked() {
            self.macros.push(("".to_string(), "".to_string()));
        }
    }

    // 5. Dictionary Page
    fn show_dictionary_page(&mut self, ui: &mut egui::Ui) {
        self.render_header(
            ui,
            "Dictionary",
            "Từ điển Tiếng Anh bỏ qua không thêm dấu & Từ điển Tiếng Việt tích hợp (Chánh Tả).",
        );

        let vn_count = vnchkey_core::vietnamese_word_set().len();
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(0x18, 0x18, 0x25))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x28, 0x2a, 0x36)))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                ui.checkbox(
                    &mut self.config.enable_dictionary,
                    "Bật User Dictionary (Tự động bỏ qua không bỏ dấu cho danh sách từ Tiếng Anh bên dưới)",
                );
                ui.add_space(6.0);
                ui.checkbox(
                    &mut self.config.spell_check,
                    format!(
                        "Bật Từ Điển Tiếng Việt Tích Hợp (Chánh Tả: {} âm tiết chuẩn OpenViet IME)",
                        vn_count
                    ),
                );
            });

        ui.add_space(12.0);

        ui.horizontal(|ui| {
            let te = egui::TextEdit::singleline(&mut self.new_dict_word)
                .hint_text("Thêm từ Tiếng Anh / thuật ngữ bỏ qua dấu...")
                .desired_width(240.0);
            let res = ui.add(te);
            let enter_pressed = res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));

            if ui.button("+ Thêm Từ").clicked() || enter_pressed {
                let trimmed = self.new_dict_word.trim().to_lowercase();
                if !trimmed.is_empty() && !self.dictionary.contains(&trimmed) {
                    self.dictionary.push(trimmed);
                    self.dictionary.sort();
                    self.new_dict_word.clear();
                }
            }

            ui.add_space(12.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.dict_search)
                    .hint_text("🔍 Tìm từ trong danh sách...")
                    .desired_width(180.0),
            );
        });
        ui.add_space(8.0);

        ui.label(
            egui::RichText::new(format!(
                "Danh sách từ Tiếng Anh bỏ qua dấu ({} từ):",
                self.dictionary.len()
            ))
            .size(12.0)
            .color(egui::Color32::from_rgb(0x89, 0xb4, 0xfa)),
        );
        ui.add_space(4.0);

        let search_filter = self.dict_search.trim().to_lowercase();
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(0x18, 0x18, 0x25))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x28, 0x2a, 0x36)))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(12))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                    let mut to_remove = None;
                    for (idx, word) in self.dictionary.iter().enumerate() {
                        if !search_filter.is_empty() && !word.to_lowercase().contains(&search_filter) {
                            continue;
                        }
                        ui.horizontal(|ui| {
                            ui.label(word);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("🗑").clicked() {
                                    to_remove = Some(idx);
                                }
                            });
                        });
                    }

                    if let Some(idx) = to_remove {
                        self.dictionary.remove(idx);
                    }
                });
            });
    }

    // 6. Keymap Page
    fn show_keymap_page(&mut self, ui: &mut egui::Ui) {
        self.render_header(ui, "Keymap", "Custom key mapping and layout configurations.");
        self.render_group_header(ui, "Bảng Quy Ước Phím Gõ");

        egui::Frame::new()
            .fill(egui::Color32::from_rgb(0x18, 0x18, 0x25))
            .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(0x28, 0x2a, 0x36)))
            .corner_radius(egui::CornerRadius::same(8))
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new(
                        "• VNI:\n\
                         1: Sắc  |  2: Huyền  |  3: Hỏi  |  4: Ngã  |  5: Nặng\n\
                         6: Dấu nón (^ cho â, ê, ô)  |  7: Dấu râu (ơ, ư)  |  8: Dấu trăng (ă)  |  9: Chữ đ\n\n\
                         • Telex:\n\
                         s: Sắc  |  f: Huyền  |  r: Hỏi  |  x: Ngã  |  j: Nặng  |  z: Xoá dấu\n\
                         aa: â  |  ee: ê  |  oo: ô  |  dd: đ  |  aw/w: ă  |  ow/w: ơ  |  uw/w: ư"
                    )
                    .monospace()
                    .size(13.0)
                    .color(egui::Color32::from_rgb(0xa6, 0xad, 0xc8)),
                );
            });
    }

    // 7. Shortcuts Page
    fn show_shortcuts_page(&mut self, ui: &mut egui::Ui) {
        self.render_header(ui, "Shortcuts", "Manage keyboard shortcuts for switching languages and modes.");
        self.render_group_header(ui, "Chuyển Đổi Nhanh");

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Phím chuyển Anh / Việt").size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("switch_shortcut")
                    .selected_text(&self.config.switch_shortcut)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.switch_shortcut, "Ctrl+Shift".to_string(), "Ctrl+Shift");
                        ui.selectable_value(&mut self.config.switch_shortcut, "Alt+Shift".to_string(), "Alt+Shift");
                        ui.selectable_value(&mut self.config.switch_shortcut, "Super+Space".to_string(), "Super+Space");
                    });
            });
        });
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Tạm thời gõ tiếng Anh khi giữ phím").size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("temp_eng_mode")
                    .selected_text(&self.config.temp_english_mode)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.temp_english_mode, "Disabled".to_string(), "Disabled");
                        ui.selectable_value(&mut self.config.temp_english_mode, "Hold Shift".to_string(), "Hold Shift");
                        ui.selectable_value(&mut self.config.temp_english_mode, "Right Alt".to_string(), "Right Alt");
                    });
            });
        });
    }

    // 8. Appearance Page
    fn show_appearance_page(&mut self, ui: &mut egui::Ui) {
        self.render_header(ui, "Appearance", "Customize the look and feel of status icons and themes.");
        self.render_group_header(ui, "Giao Diện Biểu Tượng");

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Chủ đề biểu tượng trên thanh tác vụ (Tray Icon)").size(13.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_salt("tray_theme")
                    .selected_text(&self.config.tray_icon_theme)
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.tray_icon_theme, "Default".to_string(), "Default (vnchKey)");
                        ui.selectable_value(&mut self.config.tray_icon_theme, "Breeze Dark".to_string(), "Breeze Dark");
                        ui.selectable_value(&mut self.config.tray_icon_theme, "Monochrome White".to_string(), "Monochrome White");
                        ui.selectable_value(&mut self.config.tray_icon_theme, "Minimal".to_string(), "Minimal");
                    });
            });
        });
        ui.add_space(10.0);

        ui.checkbox(
            &mut self.config.show_indicator_badge,
            "Hiển thị phù hiệu trạng thái (Status badge)",
        );
    }

    // 9. Backup Page
    fn show_backup_page(&mut self, ui: &mut egui::Ui) {
        self.render_header(ui, "Backup", "Export or restore your configuration, macros, and dictionaries.");
        self.render_group_header(ui, "Sao Lưu & Phục Hồi Dữ Liệu");

        if ui.button("📥 Xuất Sao Lưu Ra Tệp JSON (Export Backup)").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .set_file_name("vnchkey-backup.json")
                .add_filter("JSON", &["json"])
                .save_file()
            {
                if ConfigManager::export_backup(&path).is_ok() {
                    self.status_message = Some((format!("Đã xuất sao lưu ra: {}", path.display()), true));
                } else {
                    self.status_message = Some(("Không thể xuất bản sao lưu!".to_string(), false));
                }
            }
        }
        ui.add_space(10.0);

        if ui.button("📤 Phục Hồi Cài Đặt Từ Tệp JSON (Import Backup)").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .pick_file()
            {
                if ConfigManager::import_backup(&path).is_ok() {
                    self.config = ConfigManager::load_config();
                    self.macros = ConfigManager::load_macros();
                    self.dictionary = ConfigManager::load_dictionary();
                    self.app_modes = ConfigManager::load_app_modes();
                    self.status_message = Some(("Phục hồi bản sao lưu thành công!".to_string(), true));
                } else {
                    self.status_message = Some(("Không thể đọc tệp sao lưu đã chọn!".to_string(), false));
                }
            }
        }
    }

    // 10. About Page - Faithfully matching Lilypad About page screenshot
    fn show_about_page(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);

            // Centered 80x80 Logo
            if let Some(tex) = &self.logo_texture {
                ui.image(tex);
            } else {
                ui.label(egui::RichText::new("🇻🇳").size(64.0));
            }
            ui.add_space(12.0);

            // App Title - clean, regular font weight 24
            ui.label(
                egui::RichText::new("Fcitx5 vnchKey")
                    .size(24.0)
                    .color(egui::Color32::from_rgb(0xcd, 0xd6, 0xf4)),
            );
            ui.add_space(6.0);

            // Version Pill
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(0x4b, 0x44, 0x63))
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(14, 4))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("Version 0.2.0 (Stable)")
                            .size(11.0)
                            .color(egui::Color32::from_rgb(0xb4, 0xbe, 0xfe)),
                    );
                });
            ui.add_space(12.0);

            // Description
            ui.label(
                egui::RichText::new(
                    "Next-Gen Native Wayland Vietnamese IME for Linux.\n\
                     Powered by 100% Safe Rust & Sentinel Barrier."
                )
                .size(13.0)
                .color(egui::Color32::from_rgb(0xa6, 0xad, 0xc8)),
            );
            ui.add_space(6.0);

            // Link
            ui.hyperlink_to(
                egui::RichText::new("https://github.com/no-vici/fcitx5-vnchkey")
                    .size(13.0)
                    .color(egui::Color32::from_rgb(0x89, 0xb4, 0xfa)),
                "https://github.com/no-vici/fcitx5-vnchkey",
            );
            ui.add_space(16.0);

            // Action Buttons: [ Report Bug ]  [ Request Feature ]
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 415.0).max(0.0) / 2.0);
                if ui.add(egui::Button::new("Report Bug").min_size(egui::vec2(200.0, 32.0))).clicked() {
                    let _ = Command::new("xdg-open").arg("https://github.com/no-vici/fcitx5-vnchkey/issues/new").spawn();
                }
                ui.add_space(15.0);
                if ui.add(egui::Button::new("Request Feature").min_size(egui::vec2(200.0, 32.0))).clicked() {
                    let _ = Command::new("xdg-open").arg("https://github.com/no-vici/fcitx5-vnchkey/issues/new").spawn();
                }
            });
            ui.add_space(10.0);

            // Export Debug Logs Button (width 415px)
            if ui.add(egui::Button::new("Export Debug Logs").min_size(egui::vec2(415.0, 32.0))).clicked() {
                self.export_debug_logs();
            }

            ui.add_space(20.0);
            ui.separator();
            ui.add_space(12.0);

            // Credits
            ui.label(
                egui::RichText::new("Developed & Maintained by")
                    .size(13.0)
                    .color(egui::Color32::from_rgb(0x7f, 0x84, 0x9c)),
            );
            ui.add_space(4.0);

            ui.hyperlink_to(
                egui::RichText::new("no-vici (Creator & Maintainer)")
                    .size(13.0)
                    .color(egui::Color32::from_rgb(0xb4, 0xbe, 0xfe)),
                "https://github.com/no-vici",
            );
            ui.hyperlink_to(
                egui::RichText::new("Fcitx5 Input Method Framework")
                    .size(13.0)
                    .color(egui::Color32::from_rgb(0xb4, 0xbe, 0xfe)),
                "https://github.com/fcitx/fcitx5",
            );
            ui.hyperlink_to(
                egui::RichText::new("vnchKey Engine (100% Safe Rust Core)")
                    .size(13.0)
                    .color(egui::Color32::from_rgb(0xb4, 0xbe, 0xfe)),
                "https://github.com/no-vici/fcitx5-vnchkey",
            );

            ui.add_space(16.0);
            ui.label(
                egui::RichText::new("Licensed under the GNU General Public License v3.0")
                    .size(12.0)
                    .color(egui::Color32::from_rgb(0x6c, 0x70, 0x86)),
            );
        });
    }

    fn export_debug_logs(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("fcitx5-vnchkey-debug.log")
            .add_filter("Log Files", &["log", "txt"])
            .save_file()
        {
            let mut log_content = String::new();
            log_content.push_str("=== Fcitx5 vnchKey Debug Log Export ===\n");
            log_content.push_str("Version: 0.2.0 (Stable)\n");
            let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
            log_content.push_str(&format!("User: {}\n", user));
            log_content.push_str("--------------------------------------\n\n");

            log_content.push_str("--- Configuration (~/.config/fcitx5/conf/vnchkey.conf) ---\n");
            let conf_path = ConfigManager::conf_path();
            if conf_path.exists() {
                if let Ok(c) = fs::read_to_string(&conf_path) {
                    log_content.push_str(&c);
                }
            } else {
                log_content.push_str("Configuration file not found.\n");
            }
            log_content.push_str("\n\n");

            log_content.push_str("--- Recent Systemd Journal Log (fcitx5) ---\n");
            let journal = Command::new("journalctl")
                .args(["--user", "-u", "fcitx5", "--no-pager", "-n", "100"])
                .output();
            if let Ok(out) = journal {
                log_content.push_str(&String::from_utf8_lossy(&out.stdout));
            } else {
                log_content.push_str("Unable to query journalctl.\n");
            }
            log_content.push_str("\n--- End of Log ---\n");

            if fs::write(&path, log_content).is_ok() {
                self.status_message = Some((format!("Đã xuất nhật ký gỡ lỗi ra: {}", path.display()), true));
            } else {
                self.status_message = Some(("Không thể xuất nhật ký gỡ lỗi!".to_string(), false));
            }
        }
    }
}
