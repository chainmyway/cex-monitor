use anyhow::Result;
use reqwest::Client;
use tokio::time::Duration;
use serde_json::Value;
use crate::tg;

// 最新公告ID
static mut LAST_ID_BITGET: Option<String> = None;

// 检查 bitget 公告
pub async fn check_bitget() -> Result<()> {

    // Bitget 上币公告接口
    let url = "https://api.bitget.com/api/v2/public/annoucements?language=zh_CN&annType=coin_listings";

    match Client::new()
        .get(url)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        // 请求成功且状态码正常
        Ok(resp) if resp.status().is_success() => {
            let text = resp.text().await?;
            let v: Value = serde_json::from_str(&text)?;

            // 解析第一条公告
            if let Some(first) = v["data"].as_array().and_then(|arr| arr.first()) {
                    let ann_id = first["annId"].as_str().unwrap_or_default().to_string();
                    let title  = first["annTitle"].as_str().unwrap_or_default();
                    let link   = first["annUrl"].as_str().unwrap_or_default();

                    // ID 变化 有新公告
                    unsafe {
                        if Some(ann_id.clone()) != LAST_ID_BITGET {
                        // println!("🆕 Bitget 新公告:\n📄 {}\n🔗 {}", title, link);
                        if let Err(e) = tg::send_to_tg("Bitget", title, Some(link)).await {
                            eprintln!("❌ 发送到TG失败: {}", e);
                        }
                        LAST_ID_BITGET = Some(ann_id);
                    } else {
                        // println!("🙅 Bitget 无新公告");
                    }
                }
            } else {
                eprintln!("⚠️ 解析 Bitget 数据失败");
            }
        }
        // 状态码异常
        Ok(resp) => {
            eprintln!("⚠️ Bitget 状态码异常: {}", resp.status());
        }
        // 请求失败
        Err(err) => {
            eprintln!("❌ 请求 Bitget 失败: {}", err);
        }
    }

    Ok(())
}