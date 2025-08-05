use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use crate::tg;

// 记录最新公告的发布时间戳
static mut LAST_ID_BYBIT: Option<i64> = None;

// 检查 bybit 公告
pub async fn check_bybit() -> Result<()> {

    // 创建 HTTP 客户端
    let client = Client::new();

    // Bybit 公告接口，只取最新 1 条
    let resp = client
        .get("https://api.bybit.com/v5/announcements/index?type=new_crypto&limit=1&locale=zh-MY")
        .send()
        .await;

    match resp {
        Ok(res) if res.status().is_success() => {
            let body = res.text().await?;

            // 解析 JSON
            let v: Value = serde_json::from_str(&body)?;

            // 检查返回码
            if v["retCode"] == 0 {
                if let Some(obj) = v["result"]["list"].get(0) {
                    let title = obj["title"].as_str().unwrap_or("（无标题）");
                    let url = obj["url"].as_str().unwrap_or("（无链接）");
                    let publish_time = obj["publishTime"].as_i64().unwrap_or(0);

                    unsafe {
                        if Some(publish_time) != LAST_ID_BYBIT {
                            // println!("📢 Bybit 新公告:\n📄 {}\n🔗 {}", title, url);
                            if let Err(e) = tg::send_to_tg("Bybit", title, Some(url)).await {
                                eprintln!("❌ 发送到TG失败: {}", e);
                            }
                            LAST_ID_BYBIT = Some(publish_time);
                        } else {
                            // println!("🙅 Bybit 无新公告");
                        }
                    }
                } else {
                    eprintln!("⚠️ Bybit 公告列表为空或格式错误");
                }
            } else {
                eprintln!("⚠️ Bybit API retCode != 0，原始响应: {}", body);
            }
        }
        Ok(res) => {
            eprintln!("⚠️ Bybit 状态码异常: {}", res.status());
        }
        Err(err) => {
            eprintln!("❌ 请求 Bybit 失败: {}", err);
        }
    }

    Ok(())
}