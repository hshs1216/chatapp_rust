use eframe::{egui, epi};
use tokio_tungstenite::tungstenite::protocol::Message;
use std::sync::{Arc, Mutex};
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ChatApp {
    pub name: String,
    pub message: String,
    pub messages: Vec<(String, String)>,
    pub tx: Option<tokio::sync::mpsc::UnboundedSender<Message>>,
    #[allow(dead_code)]
    pub color: egui::Color32,
    pub connected: bool,
    pub name_colors: HashMap<String, egui::Color32>,
}

impl Default for ChatApp {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            name: String::new(),
            message: String::new(),
            messages: Vec::new(),
            tx: None,
            color: egui::Color32::from_rgb(rng.gen(), rng.gen(), rng.gen()),
            connected: false,
            name_colors: HashMap::new(),
        }
    }
}

impl epi::App for ChatApp {
    fn name(&self) -> &str {
        "Chat App"
    }

    fn update(&mut self, ctx: &egui::Context, _: &epi::Frame) {
        if !self.connected {
            WelcomePanel::new(&mut self.name, &mut self.connected, &mut self.tx).show(ctx);
        } else {
            egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
                InputPanel::new(&mut self.message, self.tx.clone(), &self.name).show(ui);
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                MessagePanel::new(&self.messages, &self.name_colors).show(ui);
            });
        }

        ctx.request_repaint();
    }
}

pub struct AppWrapper {
    pub app: Arc<Mutex<ChatApp>>,
}

impl epi::App for AppWrapper {
    fn name(&self) -> &str {
        "Chat App"
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let mut app = self.app.lock().unwrap();
        app.update(ctx, frame);
    }
}

pub struct WelcomePanel<'a> {
    name: &'a mut String,
    connected: &'a mut bool,
    tx: &'a mut Option<tokio::sync::mpsc::UnboundedSender<Message>>,
}

impl<'a> WelcomePanel<'a> {
    pub fn new(name: &'a mut String, connected: &'a mut bool, tx: &'a mut Option<tokio::sync::mpsc::UnboundedSender<Message>>) -> Self {
        Self { name, connected, tx }
    }

    pub fn show(mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Welcome to Chat App");
                ui.label("Enter your name to start chatting:");
                ui.text_edit_singleline(self.name);
                if ui.button("Connect").clicked() || ui.input().key_pressed(egui::Key::Enter) {
                    if !self.name.is_empty() {
                        *self.connected = true;
                        self.send_name();
                    }
                }
            });
        });
    }

    fn send_name(&mut self) {
        if let Some(tx) = &self.tx {
            let full_message = format!("{}", self.name);
            if tx.send(Message::Text(full_message)).is_err() {
                *self.tx = None; // チャンネルが閉じられている場合、txをNoneに設定
            }
        }
    }
}

pub struct MessagePanel<'a> {
    messages: &'a Vec<(String, String)>,
    name_colors: &'a HashMap<String, egui::Color32>,
}

impl<'a> MessagePanel<'a> {
    pub fn new(messages: &'a Vec<(String, String)>, name_colors: &'a HashMap<String, egui::Color32>) -> Self {
        Self { messages, name_colors }
    }

    pub fn show(self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            for (name, msg) in self.messages {
                let color = self.name_colors.get(name).cloned().unwrap_or(egui::Color32::WHITE);
                ui.colored_label(color, format!("{}: {}", name, msg));
            }
        });
    }
}

pub struct InputPanel<'a> {
    message: &'a mut String,
    tx: Option<tokio::sync::mpsc::UnboundedSender<Message>>,
    name: &'a String,
}

impl<'a> InputPanel<'a> {
    pub fn new(message: &'a mut String, tx: Option<tokio::sync::mpsc::UnboundedSender<Message>>, name: &'a String) -> Self {
        Self { message, tx, name }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            let response = ui.add_sized([ui.available_width() - 60.0, 30.0], egui::TextEdit::singleline(self.message));
            if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                self.send_message();
                response.request_focus();
            }
            if ui.button("Send").clicked() {
                self.send_message();
            }
        });
    }

    fn send_message(&mut self) {
        if let Some(tx) = &self.tx {
            if !self.message.is_empty() {
                let full_message = format!("{}: {}", self.name, self.message);
                tx.send(Message::Text(full_message)).ok();
                // 入力欄をクリア
                *self.message = String::new();
            }
        }
    }
}
