/// 历史记录结构体
pub struct History<T: Clone> {
    pub past: Vec<T>,
    pub present: T,
    pub future: Vec<T>,
    pub latest_unfiltered: T,
}

impl<T: Clone> History<T> {
    /// 创建新的历史记录
    pub fn new(
        past: Vec<T>,
        present: T,
        future: Vec<T>,
    ) -> Self {
        let latest_unfiltered = present.clone();
        History { past, present, future, latest_unfiltered }
    }
}

use crate::config::HistoryConfig;

/// 历史管理器
pub struct HistoryManager<T: Clone> {
    config: HistoryConfig,
    history: History<T>,
}

impl<T: Clone> HistoryManager<T> {
    /// 创建新的历史管理器
    pub fn new(
        initial_state: T,
        history_limit: Option<usize>,
    ) -> Self {
        let mut config = HistoryConfig::default();
        if let Some(limit) = history_limit {
            config.max_entries = limit;
        }
        Self::with_config(initial_state, config)
    }

    /// 使用配置创建历史管理器
    pub fn with_config(
        initial_state: T,
        config: HistoryConfig,
    ) -> Self {
        HistoryManager {
            config,
            history: History::new(Vec::new(), initial_state, Vec::new()),
        }
    }
    /// 获取当前状态
    pub fn get_present(&self) -> T {
        self.history.present.clone()
    }
    /// 获取历史记录
    pub fn get_history(&self) -> &History<T> {
        &self.history
    }
    /// 获取历史记录长度
    pub fn get_history_length(&self) -> usize {
        self.history.past.len() + self.history.future.len() + 1
    }

    /// 获取历史配置
    pub fn get_config(&self) -> &HistoryConfig {
        &self.config
    }

    /// 更新历史配置
    pub fn update_config(
        &mut self,
        config: HistoryConfig,
    ) {
        self.config = config;
    }
    /// 插入新状态
    ///
    /// 当历史记录超出最大限制时，自动舍弃最旧的记录
    pub fn insert(
        &mut self,
        state: T,
    ) {
        let past = &self.history.past;
        let length = past.len() + 1;

        // 处理历史长度限制：超出时舍弃第一个（最旧的）
        let past_sliced = if length >= self.config.max_entries {
            past[1..].to_vec()
        } else {
            past.clone()
        };

        // 构建新的历史记录
        let mut new_past = past_sliced;
        new_past.push(self.history.latest_unfiltered.clone());

        self.history = History::new(new_past, state.clone(), Vec::new());
        self.history.latest_unfiltered = state;
    }

    /// 跳转到未来状态
    ///
    /// # 边界检查
    /// - 检查索引是否在有效范围内
    /// - 安全的数组切片操作
    pub fn jump_to_future(
        &mut self,
        index: usize,
    ) {
        // 边界检查：确保索引有效
        if index >= self.history.future.len() {
            return;
        }

        // 安全的数组访问
        let mut new_past = self.history.past.clone();
        new_past.push(self.history.latest_unfiltered.clone());

        // 安全地添加未来状态到过去
        if index > 0 {
            new_past.extend_from_slice(&self.history.future[..index]);
        }

        let new_present = self.history.future[index].clone();

        // 安全地构建新的未来状态
        let new_future = if index + 1 < self.history.future.len() {
            self.history.future[index + 1..].to_vec()
        } else {
            Vec::new()
        };

        self.history = History::new(new_past, new_present, new_future);
    }

    /// 跳转到过去状态
    ///
    /// # 边界检查
    /// - 检查索引是否在有效范围内
    /// - 安全的数组切片操作
    pub fn jump_to_past(
        &mut self,
        index: usize,
    ) {
        // 边界检查：确保索引有效
        if index >= self.history.past.len() {
            return;
        }

        // 安全地构建新的过去状态
        let new_past = if index > 0 {
            self.history.past[..index].to_vec()
        } else {
            Vec::new()
        };

        // 安全地构建新的未来状态
        let mut new_future = Vec::new();

        // 添加被跳过的过去状态到未来
        if index + 1 < self.history.past.len() {
            new_future.extend_from_slice(&self.history.past[index + 1..]);
        }

        // 添加当前状态到未来
        new_future.push(self.history.latest_unfiltered.clone());

        // 添加原有的未来状态
        new_future.extend_from_slice(&self.history.future);

        let new_present = self.history.past[index].clone();

        self.history = History::new(new_past, new_present, new_future);
    }

    /// 通用跳转方法
    ///
    /// # 参数
    /// - `n`: 跳转的步数
    ///   - 负数：向过去跳转
    ///   - 正数：向未来跳转
    ///   - 0：不跳转
    ///
    /// # 边界检查
    /// - 自动限制在有效范围内
    /// - 超出范围的跳转会被忽略
    pub fn jump(
        &mut self,
        n: isize,
    ) {
        match n.cmp(&0) {
            std::cmp::Ordering::Less => {
                // 向过去跳转
                let past_len = self.history.past.len() as isize;
                let target = past_len + n; // n 是负数，所以这是减法

                // 边界检查：确保目标索引有效
                if target >= 0 && (target as usize) < self.history.past.len() {
                    self.jump_to_past(target as usize);
                }
                // 如果超出边界，静默忽略
            },
            std::cmp::Ordering::Equal => {
                // 不跳转
            },
            std::cmp::Ordering::Greater => {
                // 向未来跳转
                let future_index = (n - 1) as usize; // -1 因为 n=1 表示跳到第一个未来状态

                // 边界检查：确保未来索引有效
                if future_index < self.history.future.len() {
                    self.jump_to_future(future_index);
                }
                // 如果超出边界，静默忽略
            },
        }
    }

    /// 清空历史记录
    pub fn clear_history(&mut self) {
        let present = self.history.present.clone();
        self.history = History::new(Vec::new(), present, Vec::new());
    }

    /// 安全地获取过去状态
    ///
    /// # 边界检查
    /// - 返回 Option 以处理无效索引
    pub fn get_past_state(
        &self,
        index: usize,
    ) -> Option<&T> {
        self.history.past.get(index)
    }

    /// 安全地获取未来状态
    ///
    /// # 边界检查
    /// - 返回 Option 以处理无效索引
    pub fn get_future_state(
        &self,
        index: usize,
    ) -> Option<&T> {
        self.history.future.get(index)
    }

    /// 检查是否可以撤销（向过去跳转）
    pub fn can_undo(&self) -> bool {
        !self.history.past.is_empty()
    }

    /// 检查是否可以重做（向未来跳转）
    pub fn can_redo(&self) -> bool {
        !self.history.future.is_empty()
    }

    /// 获取过去状态的数量
    pub fn past_count(&self) -> usize {
        self.history.past.len()
    }

    /// 获取未来状态的数量
    pub fn future_count(&self) -> usize {
        self.history.future.len()
    }

    /// 验证历史记录的完整性
    ///
    /// 用于调试和测试，确保历史记录结构正确
    pub fn validate_integrity(&self) -> bool {
        // 检查配置是否合理
        if self.config.max_entries == 0 {
            return false;
        }

        // 检查总长度是否超出限制
        let total_length =
            self.history.past.len() + 1 + self.history.future.len();
        if total_length > self.config.max_entries {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_with_limit() {
        let mut manager = HistoryManager::with_config(
            "initial".to_string(),
            HistoryConfig { max_entries: 3, ..Default::default() },
        );

        // 插入状态，不应超出限制
        manager.insert("state1".to_string());
        manager.insert("state2".to_string());
        manager.insert("state3".to_string()); // 这应该移除 "initial"

        assert_eq!(manager.past_count(), 2); // state1, state2
        assert_eq!(manager.get_present(), "state3");
        assert!(manager.validate_integrity());
    }

    #[test]
    fn test_jump_boundary_checks() {
        let mut manager = HistoryManager::new("initial".to_string(), Some(10));

        manager.insert("state1".to_string());
        manager.insert("state2".to_string());

        // 测试跳转到过去
        manager.jump_to_past(0); // 应该跳转到 "initial"
        assert_eq!(manager.get_present(), "initial");

        // 测试无效的过去索引
        manager.jump_to_past(100); // 应该被忽略
        assert_eq!(manager.get_present(), "initial");

        // 测试跳转到未来
        manager.jump_to_future(0); // 应该跳转到 "state1"
        assert_eq!(manager.get_present(), "state1");

        // 测试无效的未来索引
        manager.jump_to_future(100); // 应该被忽略
        assert_eq!(manager.get_present(), "state1");
    }

    #[test]
    fn test_safe_access_methods() {
        let mut manager = HistoryManager::new("initial".to_string(), Some(10));
        manager.insert("state1".to_string());
        manager.insert("state2".to_string());

        // 测试安全访问
        assert_eq!(manager.get_past_state(0), Some(&"initial".to_string()));
        assert_eq!(manager.get_past_state(100), None);

        // 跳转到过去创建未来状态
        manager.jump_to_past(0);
        assert_eq!(manager.get_future_state(0), Some(&"state1".to_string()));
        assert_eq!(manager.get_future_state(100), None);
    }

    #[test]
    fn test_can_undo_redo() {
        let mut manager = HistoryManager::new("initial".to_string(), Some(10));

        // 初始状态不能撤销或重做
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());

        // 添加状态后可以撤销
        manager.insert("state1".to_string());
        assert!(manager.can_undo());
        assert!(!manager.can_redo());

        // 跳转到过去后可以重做
        manager.jump_to_past(0);
        assert!(!manager.can_undo());
        assert!(manager.can_redo());
    }

    #[test]
    fn test_jump_with_bounds() {
        let mut manager = HistoryManager::new("initial".to_string(), Some(10));
        manager.insert("state1".to_string());
        manager.insert("state2".to_string());

        // 测试通用跳转方法的边界检查
        manager.jump(-10); // 超出过去边界，应该被忽略
        assert_eq!(manager.get_present(), "state2");

        manager.jump(-1); // 跳转到 state1
        assert_eq!(manager.get_present(), "state1");

        manager.jump(10); // 超出未来边界，应该被忽略
        assert_eq!(manager.get_present(), "state1");

        manager.jump(1); // 跳转到 state2
        assert_eq!(manager.get_present(), "state2");
    }
}
