use std::collections::HashMap;

use super::rps_summary::RpsSummary;
use super::summary::Summary;

#[derive(Debug)]
pub struct Metrics {
    // 延遲
    total_latency: Summary,
    // TCP 連線時間
    tcp_connect_time: Summary,
    // TLS 握手時間
    tls_handshake_time: Summary,
    // HTTP 請求時間 (從發送請求到收到完整響應)
    http_request_time: Summary,
    // RPS
    rps_summary: RpsSummary,
    // 總錯誤數
    total_errors: usize,
    // 每秒錯誤率
    error_rates_per_sec: Summary,
    // 狀態碼計數
    status_code_counts: HashMap<u16, usize>,
    // 其他錯誤(無法獲取狀態碼的錯誤)訊息
    other_errors: Vec<String>,
}
