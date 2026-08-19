use std::time::Duration;
use eframe::egui::{self, Color32, FontId, Key, RichText, ScrollArea, Stroke};
use crate::app::state::AppState;
use crate::config::settings::Config;
use crate::database::history::HistoryDb;
use crate::predictor::context::{GitContext, ProjectType};
use crate::shell::ShellResolver;
use crate::terminal::session::TerminalSession;

pub struct TerminalApp {
    state: AppState,
    scroll_to_bottom: bool,
}

impl TerminalApp {
    pub fn new(config: Config, history_db: HistoryDb) -> Self {
        let mut state = AppState::new(config.clone(), history_db);

        // Spawn initial shell session
        let initial_shell = ShellResolver::get_best_shell(config.terminal.shell.as_deref());
        let session = match TerminalSession::new(
            format!("tab-{}", 1),
            100,
            30,
            config.terminal.scrollback_lines,
            None,
            Some(initial_shell.path.to_str().unwrap_or("")),
        ) {
            Ok(mut s) => {
                s.title = initial_shell.name.clone();
                s
            }
            Err(e) => {
                let mut s = TerminalSession::create_headless(
                    "tab-1".to_string(),
                    100,
                    30,
                    config.terminal.scrollback_lines,
                );
                s.title = "Terminal (Fallback)".to_string();
                s.feed_bytes(format!("⚠️ Shell spawn error: {}\r\n", e).as_bytes());
                s
            }
        };

        state.add_session(session);

        Self {
            state,
            scroll_to_bottom: true,
        }
    }

    pub fn run_window(config: Config, history_db: HistoryDb) -> eframe::Result<()> {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title("PredictTerm ⚡")
                .with_inner_size([1100.0, 720.0])
                .with_min_inner_size([640.0, 400.0])
                .with_decorations(true)
                .with_transparent(false),
            ..Default::default()
        };

        eframe::run_native(
            "PredictTerm",
            options,
            Box::new(|_cc| Ok(Box::new(TerminalApp::new(config, history_db)))),
        )
    }

    fn create_new_tab(&mut self) {
        let next_idx = self.state.sessions.len() + 1;
        let shell = ShellResolver::get_best_shell(self.state.config.terminal.shell.as_deref());
        let session = match TerminalSession::new(
            format!("tab-{}", next_idx),
            100,
            30,
            self.state.config.terminal.scrollback_lines,
            None,
            Some(shell.path.to_str().unwrap_or("")),
        ) {
            Ok(mut s) => {
                s.title = shell.name.clone();
                s
            }
            Err(e) => {
                let mut s = TerminalSession::create_headless(
                    format!("tab-{}", next_idx),
                    100,
                    30,
                    self.state.config.terminal.scrollback_lines,
                );
                s.title = "Terminal (Fallback)".to_string();
                s.feed_bytes(format!("⚠️ Failed to spawn shell: {}\r\n", e).as_bytes());
                s
            }
        };

        self.state.add_session(session);
    }
}

impl eframe::App for TerminalApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll PTY output every 16ms (60 FPS)
        ctx.request_repaint_after(Duration::from_millis(16));

        if let Some(session) = self.state.active_session_mut() {
            session.process_incoming();
        }

        // Tokyo Night Theme Palette
        let bg_color = Color32::from_rgb(26, 27, 38);
        let tab_bar_bg = Color32::from_rgb(22, 22, 30);
        let status_bar_bg = Color32::from_rgb(19, 20, 26);
        let text_color = Color32::from_rgb(169, 177, 214);
        let accent_color = Color32::from_rgb(122, 162, 247);
        let ghost_color = Color32::from_rgb(86, 95, 137);
        let popup_bg = Color32::from_rgb(31, 35, 53);
        let highlight_bg = Color32::from_rgb(41, 46, 66);

        // 1. Top Panel: Tab Bar & Actions
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none().fill(tab_bar_bg).inner_margin(6.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("⚡ PredictTerm").strong().color(accent_color));
                    ui.separator();

                    let mut tab_to_close = None;
                    let mut tab_to_switch = None;

                    for (idx, session) in self.state.sessions.iter().enumerate() {
                        let is_active = idx == self.state.active_session_index;
                        let tab_btn = egui::Button::new(
                            RichText::new(format!(" {} ", session.title))
                                .color(if is_active { Color32::WHITE } else { text_color })
                                .size(13.0),
                        )
                        .fill(if is_active { highlight_bg } else { Color32::TRANSPARENT })
                        .stroke(if is_active { Stroke::new(1.0_f32, accent_color) } else { Stroke::NONE });

                        if ui.add(tab_btn).clicked() {
                            tab_to_switch = Some(idx);
                        }

                        if self.state.sessions.len() > 1 {
                            if ui.small_button("×").clicked() {
                                tab_to_close = Some(idx);
                            }
                        }
                        ui.add_space(4.0);
                    }

                    if let Some(idx) = tab_to_switch {
                        self.state.switch_tab(idx);
                    }
                    if let Some(idx) = tab_to_close {
                        self.state.close_tab(idx);
                    }

                    if ui.button(RichText::new(" + ").strong().color(accent_color)).clicked() {
                        self.create_new_tab();
                    }
                });
            });

        // 2. Bottom Panel: Status Bar
        egui::TopBottomPanel::bottom("bottom_panel")
            .frame(egui::Frame::none().fill(status_bar_bg).inner_margin(4.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let active_shell = self.state.active_session().map(|s| s.title.as_str()).unwrap_or("Shell");
                    let cwd_str = self
                        .state
                        .active_session()
                        .map(|s| s.cwd.display().to_string())
                        .unwrap_or_else(|| "~".to_string());
                    let project_badge = self
                        .state
                        .active_session()
                        .and_then(|s| ProjectType::detect(&s.cwd))
                        .map(|p| format!("[{}]", p.name()))
                        .unwrap_or_default();
                    let git_badge = self
                        .state
                        .active_session()
                        .and_then(|s| GitContext::detect(&s.cwd))
                        .map(|g| format!(" {}", g.branch))
                        .unwrap_or_default();

                    ui.label(RichText::new(format!(" 🐚 {} ", active_shell)).color(Color32::from_rgb(158, 206, 106)).size(12.0));
                    ui.label(RichText::new(format!(" 📁 {} ", cwd_str)).color(text_color).size(12.0));

                    if !project_badge.is_empty() {
                        ui.label(RichText::new(project_badge).color(Color32::from_rgb(224, 175, 104)).size(12.0));
                    }
                    if !git_badge.is_empty() {
                        ui.label(RichText::new(git_badge).color(Color32::from_rgb(187, 154, 247)).size(12.0));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new("Tokyo Night | UTF-8").color(ghost_color).size(11.0));
                        ui.separator();
                        ui.label(RichText::new("AI: ON").color(accent_color).size(11.0));
                    });
                });
            });

        // 3. Central Panel: Interactive Terminal Grid & Prediction Overlay
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(bg_color).inner_margin(10.0))
            .show(ctx, |ui| {
                // Keyboard Input Interception
                let mut enter_pressed = false;
                let mut tab_pressed = false;
                let mut backspace_pressed = false;
                let mut typed_text = String::new();

                ui.input(|i| {
                    if i.key_pressed(Key::Enter) {
                        enter_pressed = true;
                    }
                    if i.key_pressed(Key::Tab) {
                        tab_pressed = true;
                    }
                    if i.key_pressed(Key::Backspace) {
                        backspace_pressed = true;
                    }

                    for event in &i.events {
                        if let egui::Event::Text(ref text) = event {
                            typed_text.push_str(text);
                        }
                    }
                });

                if let Some(session) = self.state.active_session_mut() {
                    if !typed_text.is_empty() {
                        session.input_state.insert_str(&typed_text);
                    }
                    if backspace_pressed {
                        session.input_state.backspace();
                    }
                }

                if tab_pressed {
                    let _ = self.state.accept_ghost_text().or_else(|| self.state.accept_selected_candidate());
                }

                let current_input = self
                    .state
                    .active_session()
                    .map(|s| s.input_state.text.clone())
                    .unwrap_or_default();
                let cursor_pos = self
                    .state
                    .active_session()
                    .map(|s| s.input_state.cursor_index)
                    .unwrap_or(0);

                if !typed_text.is_empty() || backspace_pressed || tab_pressed {
                    self.state.on_input_changed(current_input.clone(), cursor_pos);
                }

                if enter_pressed {
                    let _ = self.state.execute_command(current_input);
                    self.scroll_to_bottom = true;
                }

                // Render Terminal Screen Rows
                let font_id = FontId::monospace(14.0);

                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(self.scroll_to_bottom)
                    .show(ui, |ui| {
                        if let Some(session) = self.state.active_session() {
                            for line in &session.grid.lines {
                                let line_text: String = line.iter().map(|c| c.c).collect();
                                let trimmed = line_text.trim_end();
                                if !trimmed.is_empty() {
                                    ui.label(RichText::new(trimmed).font(font_id.clone()).color(text_color));
                                }
                            }
                        }

                        // Active Prompt & Input Line with Inline Ghost Text
                        let active_input_text = self
                            .state
                            .active_session()
                            .map(|s| s.input_state.text.as_str())
                            .unwrap_or("");

                        ui.horizontal(|ui| {
                            ui.label(RichText::new("❯ ").font(font_id.clone()).color(Color32::from_rgb(122, 162, 247)).strong());
                            ui.label(RichText::new(active_input_text).font(font_id.clone()).color(Color32::WHITE));

                            // Ghost Text Inline Preview
                            if let Some(pred) = &self.state.active_prediction {
                                if let Some(ref ghost) = pred.ghost_text {
                                    ui.label(RichText::new(ghost).font(font_id.clone()).color(ghost_color).italics());
                                }
                            }

                            // Blinking Cursor Block
                            let cursor_blink = (ctx.input(|i| i.time) * 2.0).fract() < 0.5;
                            if cursor_blink {
                                ui.label(RichText::new("▌").font(font_id.clone()).color(accent_color));
                            }
                        });

                        // Floating / Inline Prediction Popup
                        if let Some(pred) = &self.state.active_prediction {
                            if !pred.candidates.is_empty() {
                                ui.add_space(6.0);
                                egui::Frame::none()
                                    .fill(popup_bg)
                                    .stroke(Stroke::new(1.0_f32, accent_color))
                                    .inner_margin(8.0)
                                    .rounding(6.0)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("🔮 Suggestions [Tab to complete]").strong().color(accent_color).size(12.0));
                                        });
                                        ui.separator();

                                        for (idx, candidate) in pred.candidates.iter().take(5).enumerate() {
                                            let is_selected = idx == 0;
                                            let item_text = format!("{}. {} ", idx + 1, candidate.text);
                                            let desc = candidate
                                                .description
                                                .as_deref()
                                                .map(|d| format!("({})", d))
                                                .unwrap_or_default();

                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(item_text)
                                                        .font(font_id.clone())
                                                        .color(if is_selected { Color32::from_rgb(224, 175, 104) } else { text_color })
                                                        .strong(),
                                                );
                                                if !desc.is_empty() {
                                                    ui.label(RichText::new(desc).color(ghost_color).size(11.0));
                                                }
                                            });
                                        }
                                    });
                            }
                        }
                    });
            });
    }
}
