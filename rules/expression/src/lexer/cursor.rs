use std::cell::Cell;

/// 字符串游标结构
/// 用于在字符串中进行前后移动和字符读取操作
/// 使用Cell提供内部可变性，允许在不可变引用下修改位置
#[derive(Debug)]
pub(super) struct Cursor<'a> {
    chars: &'a [u8],      // 字符串的字节数组引用
    current: Cell<usize>, // 当前游标位置（使用Cell提供内部可变性）
}

/// 游标项类型别名
/// 包含位置索引和对应字符的元组
pub(super) type CursorItem = (usize, char);

impl<'a> From<&'a str> for Cursor<'a> {
    /// 从字符串创建游标
    /// 初始位置设为0
    fn from(source: &'a str) -> Self {
        Self { chars: source.as_bytes(), current: Cell::new(0) }
    }
}

impl Cursor<'_> {
    /// 向前移动一个字符并返回当前字符
    /// 返回 (位置, 字符) 的元组
    pub fn next(&self) -> Option<CursorItem> {
        self.advance_by(1)
    }

    /// 条件性向前移动
    /// 只有当下一个字符满足条件函数f时才移动
    pub fn next_if<F>(
        &self,
        f: F,
    ) -> Option<CursorItem>
    where
        F: Fn(char) -> bool,
    {
        let (_, c) = self.peek()?;
        if f(c) {
            return self.next();
        }

        None
    }

    /// 检查接下来的字符是否匹配给定字符串
    /// 如果匹配则消费这些字符，否则回退到原位置
    pub fn next_if_is(
        &self,
        s: &str,
    ) -> bool {
        let current = self.current.get();
        let is_valid = s.chars().all(|c| self.next_if(|ca| ca == c).is_some());
        if !is_valid {
            self.current.set(current);
        }

        is_valid
    }

    /// 向后移动一个字符并返回当前字符
    pub fn back(&self) -> Option<CursorItem> {
        self.back_by(1)
    }

    /// 查看下一个字符但不移动游标位置
    pub fn peek(&self) -> Option<CursorItem> {
        self.peek_by(1)
    }

    /// 查看上一个字符但不移动游标位置（未使用）
    #[allow(dead_code)]
    pub fn peek_back(&self) -> Option<CursorItem> {
        self.peek_back_by(1)
    }

    /// 查看向前n个位置的字符但不移动游标
    pub fn peek_by(
        &self,
        n: usize,
    ) -> Option<CursorItem> {
        self.nth(self.current.get() + n)
    }

    /// 查看向后n个位置的字符但不移动游标（未使用）
    #[allow(dead_code)]
    pub fn peek_back_by(
        &self,
        n: usize,
    ) -> Option<CursorItem> {
        self.nth(self.current.get() - n)
    }

    /// 获取当前游标位置
    pub fn position(&self) -> usize {
        self.current.get()
    }

    /// 向前移动n个字符
    pub fn advance_by(
        &self,
        n: usize,
    ) -> Option<CursorItem> {
        self.current.set(self.current.get() + n);
        self.current()
    }

    /// 向后移动n个字符
    pub fn back_by(
        &self,
        n: usize,
    ) -> Option<CursorItem> {
        self.current.set(self.current.get() - n);
        self.current()
    }

    /// 获取当前位置的字符
    pub fn current(&self) -> Option<CursorItem> {
        self.nth(self.current.get())
    }

    /// 获取指定位置n的字符
    /// 注意：这里的实现有一个off-by-one的问题，实际获取的是n-1位置的字符
    pub fn nth(
        &self,
        n: usize,
    ) -> Option<CursorItem> {
        let &c = self.chars.get(n - 1)?;
        Some((n - 1, c as char))
    }
}
