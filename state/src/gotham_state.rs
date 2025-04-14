use std::any::Any;
use std::any::TypeId;
use std::any::type_name;
use std::collections::BTreeMap;

#[derive(Default, Debug)]
pub struct GothamState {
    /// 使用BTreeMap存储不同类型的数据，以TypeId为键
    data: BTreeMap<TypeId, Box<dyn Any>>,
}

impl GothamState {
    /// 将数据存入状态容器中
    ///
    /// # 参数
    /// * `t` - 要存储的数据，必须是'static生命周期
    pub fn put<T: 'static>(
        &mut self,
        t: T,
    ) {
        let type_id = TypeId::of::<T>();
        self.data.insert(type_id, Box::new(t));
    }

    /// 检查状态容器中是否包含指定类型的数据
    ///
    /// # 参数
    /// * 泛型参数T - 要检查的类型
    ///
    /// # 返回值
    /// * 如果存在返回true，否则返回false
    pub fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.data.contains_key(&type_id)
    }

    /// 尝试获取状态容器中指定类型数据的不可变引用
    ///
    /// # 参数
    /// * 泛型参数T - 要获取的类型
    ///
    /// # 返回值
    /// * 如果存在返回Some(&T)，否则返回None
    pub fn try_borrow<T: 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.data.get(&type_id).and_then(|b| b.downcast_ref())
    }

    /// 获取状态容器中指定类型数据的不可变引用
    ///
    /// # 参数
    /// * 泛型参数T - 要获取的类型
    ///
    /// # 返回值
    /// * 返回&T，如果数据不存在则panic
    pub fn borrow<T: 'static>(&self) -> &T {
        self.try_borrow().unwrap_or_else(|| missing::<T>())
    }

    /// 尝试获取状态容器中指定类型数据的可变引用
    ///
    /// # 参数
    /// * 泛型参数T - 要获取的类型
    ///
    /// # 返回值
    /// * 如果存在返回Some(&mut T)，否则返回None
    pub fn try_borrow_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.data.get_mut(&type_id).and_then(|b| b.downcast_mut())
    }

    /// 获取状态容器中指定类型数据的可变引用
    ///
    /// # 参数
    /// * 泛型参数T - 要获取的类型
    ///
    /// # 返回值
    /// * 返回&mut T，如果数据不存在则panic
    pub fn borrow_mut<T: 'static>(&mut self) -> &mut T {
        self.try_borrow_mut().unwrap_or_else(|| missing::<T>())
    }

    /// 尝试从状态容器中移除并返回指定类型的数据
    ///
    /// # 参数
    /// * 泛型参数T - 要移除的类型
    ///
    /// # 返回值
    /// * 如果存在返回Some(T)，否则返回None
    pub fn try_take<T: 'static>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        self.data.remove(&type_id).and_then(|b| b.downcast().ok()).map(|b| *b)
    }

    /// 从状态容器中移除并返回指定类型的数据
    ///
    /// # 参数
    /// * 泛型参数T - 要移除的类型
    ///
    /// # 返回值
    /// * 返回T，如果数据不存在则panic
    pub fn take<T: 'static>(&mut self) -> T {
        self.try_take().unwrap_or_else(|| missing::<T>())
    }
}

/// 当请求的类型不存在时，生成panic错误信息
///
/// # 参数
/// * 泛型参数T - 缺失的类型
///
/// # 返回值
/// * 永不返回，总是panic
fn missing<T: 'static>() -> ! {
    panic!(
        "required type {} is not present in GothamState container",
        type_name::<T>()
    );
}

#[cfg(test)]
mod tests {
    use super::GothamState;

    struct MyStruct {
        value: i32,
    }

    struct AnotherStruct {
        value: &'static str,
    }

    type Alias1 = String;
    type Alias2 = String;

    #[test]
    fn put_borrow1() {
        let mut state = GothamState::default();
        state.put(MyStruct { value: 1 });
        assert_eq!(state.borrow::<MyStruct>().value, 1);
    }

    #[test]
    fn put_borrow2() {
        let mut state = GothamState::default();
        assert!(!state.has::<AnotherStruct>());
        state.put(AnotherStruct { value: "a string" });
        assert!(state.has::<AnotherStruct>());
        assert!(!state.has::<MyStruct>());
        state.put(MyStruct { value: 100 });
        assert!(state.has::<MyStruct>());
        assert_eq!(state.borrow::<MyStruct>().value, 100);
        assert_eq!(state.borrow::<AnotherStruct>().value, "a string");
    }

    #[test]
    fn try_borrow() {
        let mut state = GothamState::default();
        state.put(MyStruct { value: 100 });
        assert!(state.try_borrow::<MyStruct>().is_some());
        assert_eq!(state.try_borrow::<MyStruct>().unwrap().value, 100);
        assert!(state.try_borrow::<AnotherStruct>().is_none());
    }

    #[test]
    fn try_borrow_mut() {
        let mut state = GothamState::default();
        state.put(MyStruct { value: 100 });
        if let Some(a) = state.try_borrow_mut::<MyStruct>() {
            a.value += 10;
        }
        assert_eq!(state.borrow::<MyStruct>().value, 110);
    }

    #[test]
    fn borrow_mut() {
        let mut state = GothamState::default();
        state.put(MyStruct { value: 100 });
        {
            let a = state.borrow_mut::<MyStruct>();
            a.value += 10;
        }
        assert_eq!(state.borrow::<MyStruct>().value, 110);
        assert!(state.try_borrow_mut::<AnotherStruct>().is_none());
    }

    #[test]
    fn try_take() {
        let mut state = GothamState::default();
        state.put(MyStruct { value: 100 });
        assert_eq!(state.try_take::<MyStruct>().unwrap().value, 100);
        assert!(state.try_take::<MyStruct>().is_none());
        assert!(state.try_borrow_mut::<MyStruct>().is_none());
        assert!(state.try_borrow::<MyStruct>().is_none());
        assert!(state.try_take::<AnotherStruct>().is_none());
    }

    #[test]
    fn take() {
        let mut state = GothamState::default();
        state.put(MyStruct { value: 110 });
        assert_eq!(state.take::<MyStruct>().value, 110);
        assert!(state.try_take::<MyStruct>().is_none());
        assert!(state.try_borrow_mut::<MyStruct>().is_none());
        assert!(state.try_borrow::<MyStruct>().is_none());
    }

    #[test]
    fn type_alias() {
        let mut state = GothamState::default();
        state.put::<Alias1>("alias1".to_string());
        state.put::<Alias2>("alias2".to_string());
        assert_eq!(state.take::<Alias1>(), "alias2");
        assert!(state.try_take::<Alias1>().is_none());
        assert!(state.try_take::<Alias2>().is_none());
    }

    #[test]
    #[should_panic(
        expected = "required type deno_core::gotham_state::tests::MyStruct is not present in GothamState container"
    )]
    fn missing() {
        let state = GothamState::default();
        let _ = state.borrow::<MyStruct>();
    }
}
