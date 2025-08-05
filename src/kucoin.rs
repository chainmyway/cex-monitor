use anyhow::Result;
use reqwest::Client;
use crate::tg;

// 可变变量 存储最新文章ID
static mut LAST_ID_KUCOIN: Option<String> = None;

// 检查 kucoin 公告
pub async fn check_kucoin() -> Result<()> {

    let client = Client::new();
        let res = client
            .get("https://api.kucoin.com/api/v3/announcements?annType=new-listings&lang=zh_HK&pageSize=1")
            .send()
            .await;

        match res {
            Ok(resp) if resp.status().is_success() => {
                let text = resp.text().await?;
                let json: serde_json::Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    // 如果错误，则延迟一秒返回 return Ok(())，结束函数，跳过本次查询
                    Err(e) => {
                        eprintln!("❌ JSON 解析失败: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        return Ok(());
                    }
                };

                if let Some(items) = json["data"]["items"].as_array() {
                    if let Some(first) = items.first() {
                        let id = first["annId"].as_i64().unwrap_or_default().to_string();
                        let title = first["annTitle"].as_str().unwrap_or_default();
                        let url = first["annUrl"].as_str().unwrap_or_default();

                        // ID 发生变化 则有新公告
                        unsafe {
                            if Some(id.clone()) != LAST_ID_KUCOIN {
                                // println!("📢 KuCoin 新公告:\n📄 {}\n🔗 {}", title, url);
                                if let Err(e) = tg::send_to_tg("KuCoin", title, Some(url)).await {
                                    eprintln!("❌ 发送到TG失败: {}", e);
                                }
                                LAST_ID_KUCOIN = Some(id);
                            } else {
                                // println!("🙅 KuCoin 无新公告");
                            }
                        }
                    } else {
                        eprintln!("⚠️ 未找到 KuCoin 公告");
                    }
                } else {
                    eprintln!("⚠️ 返回格式错误 无 items");
                }
            }
            Ok(resp) => {
                eprintln!("⚠️ HTTP 状态错误: {}", resp.status());
            }
            Err(e) => {
                eprintln!("❌ 请求出错: {}", e);
            }
        }
    Ok(())
}