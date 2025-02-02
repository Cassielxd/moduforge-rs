use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct IdGenerator {
    data_center_id_shift: u64,
    worker_id_shift: u64,
    timestamp_left_shift: u64,
    max_data_center_id: u64,
    max_worker_id: u64,
    sequence: u64,
    last_timestamp: i64,
    options: Options,
    current_id: u64,
}

struct Options {
    start_time: i64,
    data_center_id_bits: u64,
    worker_id_bits: u64,
    sequence_bits: u64,
    worker_id: u64,
    data_center_id: u64,
}

impl IdGenerator {
    fn new() -> Self {
        let options = Options {
            start_time: 0,
            data_center_id_bits: 5,
            worker_id_bits: 5,
            sequence_bits: 12,
            worker_id: 0,
            data_center_id: 0,
        };

        let data_center_id_shift = options.worker_id_bits + options.sequence_bits;
        let worker_id_shift = options.sequence_bits;
        let timestamp_left_shift =
            options.worker_id_bits + options.sequence_bits + options.data_center_id_bits;
        let max_data_center_id = !((-1_i64 << options.data_center_id_bits) as u64);
        let max_worker_id = !((-1_i64 << options.worker_id_bits) as u64);

        IdGenerator {
            data_center_id_shift,
            worker_id_shift,
            timestamp_left_shift,
            max_data_center_id,
            max_worker_id,
            sequence: 0,
            last_timestamp: -1,
            options,
            current_id: 0,
        }
    }

    pub fn get_instance() -> &'static Mutex<IdGenerator> {
        static mut INSTANCE: Option<Mutex<IdGenerator>> = None;
        static ONCE: std::sync::Once = std::sync::Once::new();

        ONCE.call_once(|| unsafe {
            INSTANCE = Some(Mutex::new(IdGenerator::new()));
        });

        unsafe { INSTANCE.as_ref().unwrap() }
    }

    pub fn get_next_id(&mut self) -> String {
        let timestamp = self.get_timestamp();

        if timestamp < self.last_timestamp {
            panic!("Clock moved backwards");
        }

        if timestamp == self.last_timestamp {
            self.sequence = (self.sequence + 1) & self.max_sequence();
            if self.sequence == 0 {
                self.last_timestamp = self.next_millis(self.last_timestamp);
            }
        } else {
            self.sequence = 0;
        }

        self.last_timestamp = timestamp;

        self.generate_id(timestamp)
    }

    fn next_millis(&self, last_timestamp: i64) -> i64 {
        let mut timestamp = self.get_timestamp();

        while timestamp <= last_timestamp {
            timestamp = self.get_timestamp();
        }
        timestamp
    }

    fn generate_id(&self, timestamp: i64) -> String {
        let mut id = (timestamp as u128) << self.timestamp_left_shift;
        id |= (self.options.data_center_id as u128) << self.data_center_id_shift;
        id |= (self.options.worker_id as u128) << self.worker_id_shift;
        id |= self.sequence as u128;
        id.to_string()
    }

    fn get_timestamp(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        now.as_millis() as i64 - self.options.start_time
    }

    fn max_sequence(&self) -> u64 {
        !((-1_i64 << self.options.sequence_bits) as u64)
    }

    fn validate_options(&self) {
        if self.options.worker_id < 0 || self.options.worker_id > self.max_worker_id {
            panic!("Worker ID must be between 0 and {}", self.max_worker_id);
        }
        if self.options.data_center_id < 0 || self.options.data_center_id > self.max_data_center_id
        {
            panic!(
                "Data center ID must be between 0 and {}",
                self.max_data_center_id
            );
        }
    }
}
