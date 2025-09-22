/// 测试辅助工具
///
/// 提供测试中使用的错误处理宏和工具函数

/// 测试专用的 unwrap 替代宏，提供更好的错误信息
#[macro_export]
macro_rules! test_unwrap {
    ($expr:expr) => {
        $expr.expect(&format!("测试失败 [{}:{}]", file!(), line!()))
    };
    ($expr:expr, $msg:expr) => {
        $expr.expect(&format!("测试失败 [{}:{}]: {}", file!(), line!(), $msg))
    };
}

/// 测试专用的 unwrap_or_panic 宏，用于测试中的断言
#[macro_export]
macro_rules! test_assert_unwrap {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => panic!("测试断言失败 [{}:{}]: {:?}", file!(), line!(), e),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => panic!("测试断言失败 [{}:{}]: {} - {:?}", file!(), line!(), $msg, e),
        }
    };
}

/// 测试专用的期望宏，用于测试中预期的成功结果
#[macro_export]
macro_rules! test_expect_ok {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => panic!("测试期望成功但失败 [{}:{}]: {:?}", file!(), line!(), e),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => panic!("测试期望成功但失败 [{}:{}]: {} - {:?}", file!(), line!(), $msg, e),
        }
    };
}

/// 编译时检查工具函数
#[cfg(test)]
pub mod compile_time {
    use super::*;
}

/// 重新导出测试宏，使其在整个项目中可用
pub use test_unwrap;
pub use test_assert_unwrap;
pub use test_expect_ok;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwrap_macro() {
        let some_val: Option<i32> = Some(42);
        let result = test_unwrap!(some_val);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_assert_unwrap_macro() {
        let ok_result: Result<i32, &str> = Ok(42);
        let result = test_assert_unwrap!(ok_result);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_expect_ok_macro() {
        let ok_result: Result<i32, &str> = Ok(42);
        let result = test_expect_ok!(ok_result);
        assert_eq!(result, 42);
    }
}
