use nanoid::nanoid;

pub struct IdGenerator;

impl IdGenerator {
    pub fn get_id() -> String {
        // 使用数字和大写字母，生成12位的ID
        // 使用自定义字符集：数字(0-9)和大写字母(A-Z)，去掉容易混淆的字符
        const ALPHABET: [char; 32] = [
            '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K',
            'L', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'U', 'V',
            'W', 'X'
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
}
