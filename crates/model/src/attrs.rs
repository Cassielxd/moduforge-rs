use std::ops::{Deref, DerefMut};
use std::ops::{Index, IndexMut};
use imbl::HashMap;
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use serde_json::Value;
//pub type Attrs = HashMap<String, Value>;
/// 节点属性
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attrs {
    pub attrs: HashMap<String, Value>,
}

impl Serialize for Attrs {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.attrs.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Attrs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<String, Value>::deserialize(deserializer)?;
        Ok(Attrs { attrs: map })
    }
}

impl Default for Attrs {
    fn default() -> Self {
        Self { attrs: HashMap::new() }
    }
}

impl Index<&str> for Attrs {
    type Output = Value;

    fn index(
        &self,
        key: &str,
    ) -> &Self::Output {
        self.get_safe(key).expect("Key not found")
    }
}

// 实现 IndexMut trait 用于修改值
impl IndexMut<&str> for Attrs {
    fn index_mut(
        &mut self,
        key: &str,
    ) -> &mut Self::Output {
        if !self.attrs.contains_key(key) {
            self.attrs.insert(key.to_string(), Value::Null);
        }
        self.attrs.get_mut(key).expect("Key not found")
    }
}

impl Attrs {
    pub fn from(new_values: HashMap<String, Value>) -> Self {
        Self { attrs: new_values }
    }
    pub fn get_value<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Option<T> {
        self.attrs.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    pub fn update(
        &self,
        new_values: HashMap<String, Value>,
    ) -> Self {
        let mut attrs = self.attrs.clone();
        for (key, value) in new_values {
            attrs.insert(key, value);
        }
        Attrs { attrs }
    }
    pub fn get_safe(
        &self,
        key: &str,
    ) -> Option<&Value> {
        self.attrs.get(key)
    }
}

impl Deref for Attrs {
    type Target = HashMap<String, Value>;

    fn deref(&self) -> &Self::Target {
        &self.attrs
    }
}

impl DerefMut for Attrs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.attrs
    }
}
