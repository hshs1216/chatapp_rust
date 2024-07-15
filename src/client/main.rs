mod app;
mod connection;

use app::{AppWrapper, ChatApp};
use std::sync::{Arc, Mutex};
use eframe::NativeOptions;

#[tokio::main]
async fn main() {
    let app = Arc::new(Mutex::new(ChatApp::default()));
    let app_clone = Arc::clone(&app);

    tokio::spawn(async move {
        connection::connect(app_clone).await;
    });

    let native_options = NativeOptions::default();
    eframe::run_native(
        Box::new(AppWrapper { app }),
        native_options,
    );
}
