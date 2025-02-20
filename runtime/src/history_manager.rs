/// 历史记录结构体
pub struct History<T: Clone> {
    past: Vec<T>,
    present: T,
    future: Vec<T>,
    latest_unfiltered: T,
}

impl<T: Clone> History<T> {
    /// 创建新的历史记录
    fn new(past: Vec<T>, present: T, future: Vec<T>) -> Self {
        let latest_unfiltered = present.clone();
        History {
            past,
            present,
            future,
            latest_unfiltered,
        }
    }
}

/// 历史管理器
pub struct HistoryManager<T: Clone> {
    limit: Option<usize>,
    history: History<T>,
}

impl<T: Clone> HistoryManager<T> {
    /// 创建新的历史管理器
    pub fn new(initial_state: T, history_limit: Option<usize>) -> Self {
        HistoryManager {
            limit: history_limit,
            history: History::new(Vec::new(), initial_state, Vec::new()),
        }
    }

    pub fn get_present(&self) -> T {
        self.history.present.clone()
    }

    /// 插入新状态
    pub fn insert(&mut self, state: T) {
        let past = &self.history.past;
        let length = past.len() + 1;

        // 处理历史长度限制
        let past_sliced = match self.limit {
            Some(limit) if length >= limit => past[1..].to_vec(),
            _ => past.clone(),
        };

        // 构建新的历史记录
        let mut new_past = past_sliced;
        new_past.push(self.history.latest_unfiltered.clone());

        self.history = History::new(new_past, state, Vec::new());
    }

    /// 跳转到未来状态
    pub fn jump_to_future(&mut self, index: usize) {
        if index >= self.history.future.len() {
            return;
        }

        let mut new_past = self.history.past.clone();
        new_past.push(self.history.latest_unfiltered.clone());
        new_past.extend_from_slice(&self.history.future[..index]);

        let new_present = self.history.future[index].clone();
        let new_future = self.history.future[index + 1..].to_vec();

        self.history = History::new(new_past, new_present, new_future);
    }

    /// 跳转到过去状态
    pub fn jump_to_past(&mut self, index: usize) {
        if index >= self.history.past.len() {
            return;
        }

        let new_past = self.history.past[..index].to_vec();
        let mut new_future = self.history.past[index + 1..].to_vec();
        new_future.push(self.history.latest_unfiltered.clone());
        new_future.extend_from_slice(&self.history.future);

        let new_present = self.history.past[index].clone();

        self.history = History::new(new_past, new_present, new_future);
    }

    /// 通用跳转方法
    pub fn jump(&mut self, n: isize) {
        match n.cmp(&0) {
            std::cmp::Ordering::Less => {
                let past_len = self.history.past.len() as isize;
                let target = past_len + n;
                if target >= 0 {
                    self.jump_to_past(target as usize);
                }
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => {
                self.jump_to_future((n - 1) as usize);
            }
        }
    }

    /// 清空历史记录
    pub fn clear_history(&mut self) {
        let present = self.history.present.clone();
        self.history = History::new(Vec::new(), present, Vec::new());
    }
}
