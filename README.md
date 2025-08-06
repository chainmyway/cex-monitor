# cex-monitor
监控六个主流数字货币交易所的上币公告：Gate Bybit Bitget KuCoin Binance OKX 

在线文档 [https://chainmyway.com ](https://www.chainmyway.com/cex_monitor_notify/00-%E5%89%8D%E8%A8%80)

还有对应的教学视频
- B站 https://www.bilibili.com/video/BV1E3tnzCE5W
- Youtube https://www.youtube.com/watch?v=tmvrUiPIlnA&list=PLMsVemdMJKNXCBgj5KD2vqbP-_Sdks0Ow&index=1

请填入以下参数
- tg.rs 模块
  - Bot Token -> TELEGRAM_BOT_TOKEN
  - 群组ID -> TELEGRAM_CHAT_ID
  - 子话题ID，若无开启话题，可以忽略 -> TELEGRAM_TOPIC_ID

- binance_wss.rs 模块，申请BN API获取
  - API key -> api_key
  - API secret -> api_secret
