use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use crate::app::ChatApp;
use rand::Rng;

pub async fn connect(app: Arc<Mutex<ChatApp>>) {
    let url = "ws://127.0.0.1:8080";
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    {
        let mut app = app.lock().unwrap();
        app.tx = Some(tx);
    }

    let app_clone = Arc::clone(&app);
    tokio::spawn(async move {
        while let Some(message) = read.next().await {
            if let Ok(Message::Text(text)) = message {
                let mut app = app_clone.lock().unwrap();
                if text.starts_with("--") && text.ends_with("--") {
                    // This is a join or exit message
                    app.messages.push(("System".to_string(), text.clone()));
                } else {
                    let parts: Vec<&str> = text.splitn(2, ": ").collect();
                    if parts.len() == 2 {
                        let name = parts[0].to_string();
                        let msg = parts[1].to_string();
                        app.messages.push((name.clone(), msg));

                        // Assign a color if not already assigned
                        if !app.name_colors.contains_key(&name) {
                            let mut rng = rand::thread_rng();
                            let color = egui::Color32::from_rgb(rng.gen(), rng.gen(), rng.gen());
                            app.name_colors.insert(name, color);
                        }
                    }
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if write.send(message).await.is_err() {
                break;
            }
        }
    });
}
