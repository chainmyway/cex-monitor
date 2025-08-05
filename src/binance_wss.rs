use chrono::Utc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use anyhow::Result;
use ring::hmac;
use uuid::Uuid;
use crate::tg;

static mut LAST_PUBLISHED: Option<u64> = None;

fn sign_query(params: &str, secret: &str) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, secret.as_bytes());
    let tag = hmac::sign(&key, params.as_bytes());
    tag.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
}

pub async fn check_binance_wss() -> Result<()> {
    let api_key = "API key";
    let api_secret = "API secret";

    loop {
        let timestamp = Utc::now().timestamp_millis();
        let random = Uuid::new_v4().simple().to_string();
        let topic = "com_announcement_en";
        let recv_window = 30000;
        
        // 排序参数
        let params_for_signature = format!(
            "random={}&recvWindow={}&timestamp={}&topic={}",
            random, recv_window, timestamp, topic
        );
        
        let signature = sign_query(&params_for_signature, api_secret);
        
        // 构建最终 URL
        let final_params = format!(
            "random={}&recvWindow={}&timestamp={}&topic={}&signature={}",
            random, recv_window, timestamp, topic, signature
        );

        let url: String = format!("wss://api.binance.com/sapi/wss?{}", final_params);

        // 添加自定义头
        let url_parsed = url::Url::parse(&url)?;
        let mut request = url_parsed.into_client_request()?;
        let headers = request.headers_mut();
        headers.insert("X-MBX-APIKEY", api_key.parse()?);

        match connect_async(request).await {
            Ok((ws_stream, response)) => {
                println!("✅ Connected, HTTP status: {}", response.status());
                let (write_raw, mut read) = ws_stream.split();
                let write = Arc::new(Mutex::new(write_raw));
                let stop_ping = Arc::new(tokio::sync::Notify::new());

                // 发送订阅请求
                let sub = serde_json::json!({
                    "command": "SUBSCRIBE",
                    "value": topic
                });
                
                if let Err(e) = write.lock().await.send(Message::Text(sub.to_string())).await {
                    eprintln!("❌ 发送订阅请求失败 {}", e);
                    continue;
                }

                // 每30秒发送一次PING
                let w_ping = write.clone();
                let stop_ping_clone = stop_ping.clone();
                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = stop_ping_clone.notified() => {
                                break;
                            }
                            _ = async {
                                if let Err(e) = w_ping.lock().await.send(Message::Ping(Vec::new())).await {
                                    eprintln!("⚠️ Ping 错误: {}", e);
                                }
                                sleep(Duration::from_secs(30)).await;
                            } => {}
                        }
                    }
                });

                // 消息循环
                while let Some(msg) = read.next().await {
                    match msg {
                        Ok(Message::Text(txt)) => {
                            
                            if let Ok(v) = serde_json::from_str::<Value>(&txt) {
                                
                                // 数据消息
                                if let Some(msg_type) = v.get("type").and_then(|x| x.as_str()) {
                                    
                                    // 根据消息类型处理不同的数据
                                    match msg_type {

                                        "COMMAND" => {

                                            println!("📋 订阅币安结果 {}", txt, );
                                            
                                        }

                                        "DATA" => {
                                            if let Some(data_str) = v.get("data").and_then(|x| x.as_str()) {
                                                if let Ok(inner_data) = serde_json::from_str::<Value>(data_str) {
                                                    if let Some(ts) = inner_data.get("publishDate").and_then(|x| x.as_u64()) {
                                                        unsafe {
                                                            if LAST_PUBLISHED.map_or(true, |prev| prev != ts) {
                                                                LAST_PUBLISHED = Some(ts);

                                                                let title_outer = inner_data.get("title").and_then(|x| x.as_str()).unwrap_or("无标题");
                                                                let catalog_id = inner_data.get("catalogId").and_then(|x| x.as_u64()).unwrap_or(0);

                                                                if catalog_id == 48 {
                                                                    let title = inner_data.get("title").and_then(|x| x.as_str()).unwrap_or(title_outer);

                                                                    // println!("🚨 新公告: {}", title);

                                                                    if let Err(e) = tg::send_to_tg("币安", title, None).await {
                                                                        eprintln!("❌ 发送到TG失败: {}", e);
                                                                    }
                                                                } else {
                                                                    // let title = inner_data.get("title").and_then(|x| x.as_str()).unwrap_or(title_outer);

                                                                    // println!("🚨 非上币公告: catalogId = {}", catalog_id);
                                                                    // println!("内容: {}", title);
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    eprintln!("❌ 解析 data 失败: {}", data_str);
                                                }
                                            } else {
                                                println!("⚠️ 没有找到 data 、格式不对");
                                            }
                                        }
                                        _ => {
                                            println!("📋 其他类型消息: {}", msg_type);
                                        }
                                    }
                                } else {
                                    println!("📋 未知格式消息: {}", serde_json::to_string_pretty(&v).unwrap_or_else(|_| "Parse error".to_string()));
                                }
                            } else {
                                println!("❌ 解析 JSON 失败: {}", txt);
                            }
                        }
                        Ok(Message::Ping(p)) => {
                            println!("🏓 Received ping from server, sending pong");
                            write.lock().await.send(Message::Pong(p)).await?;
                        }

                        // 每 30 秒发送一次 Ping
	                    // BN 收到 Ping ，回复 Pong
                        Ok(Message::Pong(_)) => {
                            // println!("🏓 收到了Pong");
                        }

                        Ok(Message::Close(_)) => {
                            println!("🔌 连接关闭");
                            break;
                        }

                        Err(e) => {
                            eprintln!("❌ WebSocket 错误: {}", e);
                            break;
                        }

                        _ => {
                            println!("📨 获得其它 type 消息");
                        }
                    }
                }
                
                stop_ping.notify_waiters();
                println!("🔄 重新连接");
            }

            Err(e) => {
                eprintln!("❌ 报错 {}", e);
            }

        }

        // 断开5秒重连
        sleep(Duration::from_secs(5)).await;
    }
}