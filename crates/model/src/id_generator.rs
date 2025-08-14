use uuid::Uuid;
use base62::encode;

pub struct IdGenerator;

impl IdGenerator {
    pub fn get_id() -> Box<str>  {
        let uuid = Uuid::new_v4();
        let num = u128::from_be_bytes(*uuid.as_bytes());
        encode(num).into_boxed_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    #[test]
    fn test_id_generation() {
        let _id = IdGenerator::get_id();
        println!("生成新ID: {:?}", _id);
    }
    #[test]
    fn test_id_generation_performance() {
        const ITERATIONS: usize = 1_000_000;

        // 测试新ID生成器性能
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _id = IdGenerator::get_id();
        }
        let new_duration = start.elapsed();

        println!("生成 {} 个新ID耗时: {:?}", ITERATIONS, new_duration);
        println!("平均每个ID生成时间: {:?}", new_duration / ITERATIONS as u32);
    }

    #[test]
    fn test_id_uniqueness() {
        const ITERATIONS: usize = 1_000_000;
        let mut ids = std::collections::HashSet::with_capacity(ITERATIONS);

        // 生成大量ID并检查唯一性
        for _ in 0..ITERATIONS {
            let id = IdGenerator::get_id();
            assert!(ids.insert(id), "发现重复ID！");
        }

        println!("成功生成 {} 个唯一ID", ITERATIONS);

        // 计算碰撞概率
        let total_possible = 32u64.pow(12);
        let collision_probability =
            1.0 - (1.0 - 1.0 / (total_possible as f64)).powi(ITERATIONS as i32);
        println!("理论碰撞概率: {:.10}%", collision_probability * 100.0);
    }
}
