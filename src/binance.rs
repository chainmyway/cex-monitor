use anyhow::{Result, anyhow};
use reqwest::Client;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::tg;

// Binance 公告接口
const BINANCE_URL: &str = "https://www.binance.com/bapi/composite/v1/public/cms/article/catalog/list/query";

// 可变变量 存储最新文章ID
static mut LAST_ID_BINANCE: Option<u64> = None;

// 检查 Binance 公告
pub async fn check_binance(client: &Client) -> Result<()> {

    // 接口参数
    // catalogId=48 只查询 交易对上新 分区的新闻
    // pageNo=1 从第一页开始
    // pageSize=1 每页只查询1条数据
    let url = format!("{}?catalogId=48&pageNo=1&pageSize=1", BINANCE_URL);

    // 生成随机值，降低 CDN 缓存影响
    // SystemTime::now() 当前系统时间
    // UNIX_EPOCH 常量，表示 1970年1月1日 00:00:00 的时间戳
    // duration_since(UNIX_EPOCH) 自 UNIX_EPOCH 以来的时间差
    // unwrap 计算时若有错误，直接 panic 崩溃程序
    // as_millis() 转换成毫秒数
    // % 1000 取模，生成 0-999 的随机数
    let ua = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() % 1000;

    // 发送请求
    // 并使用 match 匹配返回值的几种可能
    match client
            .get(&url)

            // 设置随机请求头，模拟真实用户访问
            // 64 位 Windows
            // 浏览器内核 WebKit
            // Chrome浏览器，版本号为 随机值ua
            .header(
                "User-Agent",
                &format!(
                    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
                    ua
                ),
            )
            .header("Accept-Encoding", "gzip")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", "https://www.binance.com/en/support/announcement")
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            
            // 发送
            .send()
            
            // 异步等待 请求结果
            .await

            // 匹配请求结果，三种可能性
            {
                // 请求成功，且状态码正常（200～299）
                Ok(resp) if resp.status().is_success() => {
                    let text = resp.text().await?;
                    
                    // 解析 JSON
                    let json: serde_json::Value = serde_json::from_str(&text)?;
                    // 检查 JSON 是否包含 data、 articles
                    if let Some(first_article) = json["data"]["articles"].get(0) {
                        
                        // 获取文章 ID 和标题
                        let id = first_article["id"].as_u64().ok_or_else(|| anyhow!("缺失公告ID"))?;
                        let title = first_article["title"].as_str().ok_or_else(|| anyhow!("缺失公告标题"))?;

                        // 检查是否有新公告
                        unsafe {
                            if Some(id) != LAST_ID_BINANCE {
                                // println!("🆕 Binance 新公告:\n📄 {}", title);
                                if let Err(e) = tg::send_to_tg("币安",title,None)
                                    .await {
                                        eprintln!("❌ 发送到TG失败: {}", e);
                                    }
                                LAST_ID_BINANCE = Some(id);
                            } else {
                                // println!("🙅 Binance 无新公告");
                            }
                        }
                    } else {
                        return Err(anyhow!("未查询到文章"));
                    }
                }

                // 请求成功，但状态码异常（400、403等）
                // 一般是发送请求成功，但IP被BN封禁
                Ok(resp) => {
                    return Err(anyhow!("状态码异常: {}", resp.status()));
                }

                // 请求失败
                // 一般是网络错误、DNS解析失败等，本机错误
                Err(err) => {
                    return Err(anyhow!("请求失败: {}", err));
                }
            }

    Ok(())
}