// mod ipidea;
// mod binance;
mod okx;
mod bitget;
mod bybit;
mod gate;
mod kucoin;
mod tg;
mod binance_wss;

use anyhow::Result;
// use reqwest::{Client, Proxy};
use tokio::time::{sleep, Duration};

// 主函数
#[tokio::main]
async fn main() -> Result<()> {
    
    // 每个 CEX 单独一个任务（协程），互不影响
    tokio::spawn(async {
        if let Err(e) = binance_wss::check_binance_wss().await {
            eprintln!("❌ Binance-wss 停止: {}", e);
        }
    });
    
    // tokio::spawn(async {
    //     if let Err(e) = binance_monitor().await {
    //         eprintln!("❌ Binance-轮询 监听器退出: {}", e);
    //     }
    // });

    tokio::spawn(async {
        if let Err(e) = okx_monitor().await {
            eprintln!("❌ OKX 监听器退出: {}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = bitget_monitor().await {
            eprintln!("❌ Bitget 监听器退出: {}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = bybit_monitor().await {
            eprintln!("❌ Bybit 监听器退出: {}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = gate_monitor().await {
            eprintln!("❌ gate 监听器退出: {}", e);
        }
    });

    tokio::spawn(async {
        if let Err(e) = kucoin_monitor().await {
            eprintln!("❌ kucoin 监听器退出: {}", e);
        }
    });

    tokio::signal::ctrl_c().await?;
    println!("🛑 主程序结束");

    Ok(())
}


// Binance 监听
// async fn binance_monitor() -> Result<()> {

//     // 获取代理IP
//     let mut proxy = ipidea::fetch_proxy().await?;

//     // 循环发送请求
//     loop {
//         println!("🌐 当前使用代理: {}", proxy);

//         // 构建使用代理的 HTTP 客户端
//         // 使用 match 匹配 ok、err 两种结果
//         let client = match Client::builder()
//             .proxy(Proxy::all(&proxy)?)
//             .timeout(Duration::from_secs(10))
//             .build()
//         {
//             Ok(c) => c,
//             Err(e) => {
//                 eprintln!("❌ 构建客户端失败: {}，尝试更换代理", e);
//                 proxy = ipidea::fetch_proxy().await?;
//                 continue;
//             }
//         };
        
//         // 查询BN公告
//         // 如果返回异常，则更换IP
//         if let Err(err) = binance::check_binance(&client).await {
//             eprintln!("❌ 请求Binance失败: {}, 正在更换IP", err);
//             proxy = ipidea::fetch_proxy().await?;
//         }
        
//         // 每10秒查询一次
//         sleep(Duration::from_secs(10)).await;
//     }
// }

// OKX 监听
async fn okx_monitor() -> Result<()> {

    loop {
        // 查询OKX公告
        if let Err(err) = okx::check_okx().await {
            eprintln!("❌ 请求OKX失败: {}", err);
        }

        // 5秒查询一次
        sleep(Duration::from_secs(5)).await;
    }

}

// Bitget 监听
async fn bitget_monitor() -> Result<()> {
    loop {
        // 查询Bitget公告
        if let Err(err) = bitget::check_bitget().await {
            eprintln!("❌ 请求Bitget失败: {}", err);
        }

        // 5秒查询一次
        sleep(Duration::from_secs(5)).await;
    }
}

// Bybit 监听
async fn bybit_monitor() -> Result<()> {
    loop {
        // 查询Bybit公告
        if let Err(err) = bybit::check_bybit().await {
            eprintln!("❌ 请求Bybit失败: {}", err);
        }

        // 5秒查询一次
        sleep(Duration::from_secs(5)).await;
    }
}

// Gate 监听
async fn gate_monitor() -> Result<()> {
    // 虽然是 wss，但依然轮询，防止断开
    loop {
        // 查询Gate公告
        if let Err(err) = gate::check_gate().await {
            eprintln!("❌ 请求Gate失败: {}", err);
        }

        // 10秒查询一次
        sleep(Duration::from_secs(10)).await;
    }
}

// KuCoin 监听
async fn kucoin_monitor() -> Result<()> {

    loop {
        // 查询KuCoin公告
        if let Err(err) = kucoin::check_kucoin().await {
            eprintln!("❌ 请求KuCoin失败: {}", err);
        }

        // 5秒查询一次
        sleep(Duration::from_secs(5)).await;
    }
}