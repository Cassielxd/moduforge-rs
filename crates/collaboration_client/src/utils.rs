use std::time::{SystemTime, UNIX_EPOCH};

/// 获取当前时间戳（毫秒）
pub fn get_unix_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
        as u64
}
