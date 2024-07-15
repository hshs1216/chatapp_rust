use tokio::net::TcpStream;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::tungstenite::Message;
use tokio::sync::{Mutex, mpsc};
use tokio::sync::mpsc::UnboundedSender;
use std::sync::Arc;

type Tx = UnboundedSender<Message>;

pub async fn handle_connection(stream: TcpStream, clients: Arc<Mutex<Vec<Tx>>>) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let (tx, mut rx) = mpsc::unbounded_channel();
    {
        let mut clients_guard = clients.lock().await;
        clients_guard.push(tx.clone());
    }

    // 名前を取得
    let username = match ws_receiver.next().await {
        Some(Ok(Message::Text(name))) => name,
        _ => {
            ws_sender.close().await?;
            return Ok(());
        }
    };
    println!("Username is: {}", username);

    // 入室メッセージを送信
    let join_message = Message::Text(format!("-- {} is entering. --", username));
    {
        let clients_guard = clients.lock().await;
        for client in &*clients_guard {
            client.send(join_message.clone()).unwrap_or_else(|_| ());
        }
    }

    // 別のタスクで受信したメッセージをクライアントに送信
    let clients_clone = Arc::clone(&clients);
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // メインタスクでメッセージを受信してブロードキャスト
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(msg) => {
                let clients_guard = clients.lock().await;
                for client in &*clients_guard {
                    client.send(msg.clone()).unwrap_or_else(|_| ());
                }
            }
            Err(_) => {
                // 接続が切れた場合
                let exit_message = Message::Text(format!("-- {} is exited. --", username));
                let mut clients_guard = clients.lock().await;
                clients_guard.retain(|client| !same_channel(client, &tx));
                for client in &*clients_guard {
                    client.send(exit_message.clone()).unwrap_or_else(|_| ());
                }
                break;
            }
        }
    }

    // クライアントを削除
    let mut clients_guard = clients.lock().await;
    clients_guard.retain(|client| !same_channel(client, &tx));

    Ok(())
}

// `same_channel` 関数を実装
fn same_channel<T>(sender1: &mpsc::UnboundedSender<T>, sender2: &mpsc::UnboundedSender<T>) -> bool {
    std::ptr::eq(sender1, sender2)
}
