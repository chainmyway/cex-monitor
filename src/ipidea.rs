use anyhow::{anyhow, Result};

// ipidea API
const PROXY_API: &str = "http://api.proxy.ipidea.io/getProxyIp?num=1&return_type=txt&lb=1&protocol=http";

// 从 ipidea 获取代理 IP
pub async fn fetch_proxy() -> Result<String> {
    println!("🔄 正在从 ipidea 获取代理...");

    // 发送get请求 读取相应结果
    let raw = reqwest::get(PROXY_API).await?.text().await?;

    // 检查是否为 JSON 格式（报错信息）
    if raw.trim_start().starts_with('{') {
        return Err(anyhow!("ipidea 返回错误: {}", raw));
    }

    // 解析返回结果
    let proxy = raw
        // 按行分割
        .lines()
        // 过滤空行
        .find(|l| !l.trim().is_empty())
        // 若无有效行则报错
        .ok_or_else(|| anyhow!("ipidea 返回空代理"))?
        // 去除前后空格
        .trim()
        // 转换成字符串
        .to_string();

    println!("✅ 获取到代理: {}", proxy);

    // 返回IP
    Ok(proxy)
}