use nanoid::nanoid;

pub struct IdGenerator;

impl IdGenerator {
    pub fn get_id() -> String {
        // 生成一个21位的唯一ID
        nanoid!()
    }
}
