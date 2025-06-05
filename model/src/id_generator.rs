use nanoid::nanoid;

use std::collections::HashSet;
pub struct IdGenerator;

impl IdGenerator {
    pub fn get_id() -> String {
        // 使用数字和大写字母，生成12位的ID
        // 使用自定义字符集：数字(0-9)和大写字母(A-Z)，去掉容易混淆的字符
        const ALPHABET: [char; 32] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
            'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'j', 'k',
            'l', 'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v',
            'w', 'x'
        ];
        
        nanoid!(12, &ALPHABET)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

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
        let mut ids = HashSet::with_capacity(ITERATIONS);
        
        // 生成大量ID并检查唯一性
        for _ in 0..ITERATIONS {
            let id = IdGenerator::get_id();
            assert!(ids.insert(id), "发现重复ID！");
        }
        
        println!("成功生成 {} 个唯一ID", ITERATIONS);
        
        // 计算碰撞概率
        let total_possible = 32u64.pow(12);
        let collision_probability = 1.0 - (1.0 - 1.0/(total_possible as f64)).powi(ITERATIONS as i32);
        println!("理论碰撞概率: {:.10}%", collision_probability * 100.0);
    }
}
