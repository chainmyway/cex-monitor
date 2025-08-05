use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use anyhow::Result;
use crate::tg;

// 上次公告time
static mut LAST_GATE_PUBLISHED: Option<u64> = None;

pub async fn check_gate() -> Result<()> {
    
    // 建立 WebSocket 连接
    match connect_async("wss://api.gateio.ws/ws/v4/ann").await {
        Ok((ws_stream, _)) => {
            
            let (write_raw, mut read) = ws_stream.split();
            let write = Arc::new(Mutex::new(write_raw));

            // 发送订阅“上币公告”请求
            let sub_msg = serde_json::json!({
                "time": Utc::now().timestamp(),
                "channel": "announcement.summary_listing",
                "event": "subscribe",
                "payload": ["cn"]
            });
            {
                // 加锁后写入
                let mut w = write.lock().await;
                w.send(Message::Text(sub_msg.to_string().into())).await?;
            }

            // println!("✅ 请求订阅Gate");

            // 主动 Ping ：每 15 秒发送一次保持连接
            let write_ping = Arc::clone(&write);
            tokio::spawn(async move {
                loop {
                    if let Err(e) = write_ping.lock().await.send(Message::Ping(Vec::new())).await {
                        eprintln!("⚠️ 主动 Ping Gate 失败: {e}");
                        break;
                    }
                    sleep(Duration::from_secs(15)).await;
                }
            });


            // 处理消息
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {

                        let v: Value = serde_json::from_str(&text).unwrap_or_default();

                        // 成功订阅
                        if v.get("event") == Some(&Value::String("subscribe".into())) {
                            if let Some(result) = v.get("result") {
                                if result.get("status") == Some(&Value::String("success".into())) {
                                    println!("📡 订阅Gate成功: {}", v["channel"]);
                                }
                            }
                        }

                        // 处理公告
                        if v.get("channel") == Some(&Value::String("announcement.summary_listing".into())) {
                            if let Some(result) = v.get("result") {
                                if let Some(published) = result.get("published_at").and_then(|v| v.as_u64()) {
                                    unsafe {
                                        // 检查新公告
                                        if LAST_GATE_PUBLISHED.map_or(true, |prev| prev != published) {
                                            LAST_GATE_PUBLISHED = Some(published);
                                            let title = result.get("title").and_then(|v| v.as_str()).unwrap_or("（Gate公告 无标题）");
                                            // println!("🆕 Gate 上币公告:\n📄 {}", title);
                                            if let Err(e) = tg::send_to_tg("Gate", title, None).await {
                                                eprintln!("❌ 发送到TG失败: {}", e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 服务器 Ping → 立即回 Pong 消息
                    // 以免断开连接
                    // 既会主动 Ping
                    // 也会被动回 Pong（收到 Gate Ping 时）
                    Ok(Message::Ping(payload)) => {
                        let mut w = write.lock().await;
                        w.send(Message::Pong(payload)).await?;
                    }

                    // 服务器关闭 → 跳出循环，外层重连
                    Ok(Message::Close(_)) => {
                        println!("🔌 Gate 连接关闭，5 秒后重连…");
                        break;
                    }

                    // 其它错误
                    Err(e) => {
                        eprintln!("❌ Gate WebSocket 错误: {e}");
                        break;
                    }

                    _ => {}
                }
            }
        }

        // 连接建立失败
        Err(e) => eprintln!("❌ Gate WebSocket 连接失败: {e}"),
    }

    Ok(())
}