use std::ops::{Deref, DerefMut};
use std::ops::{Index, IndexMut};
use im::HashMap;
use serde::{Deserialize, Serialize, Deserializer, Serializer};
use serde_json::Value;
//pub type Attrs = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

/// 用于选择性序列化 Attrs 的包装器
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FilteredAttrs<'a> {
    attrs: &'a Attrs,
    filter_key: &'a str,
}

impl<'a> FilteredAttrs<'a> {
    pub fn new(
        attrs: &'a Attrs,
        filter_key: &'a str,
    ) -> Self {
        Self { attrs, filter_key }
    }
}

impl<'a> Serialize for FilteredAttrs<'a> {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serde_json::Map::new();
        if let Some(value) = self.attrs.get_safe(self.filter_key) {
            map.insert(self.filter_key.to_string(), value.clone());
        }
        map.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_attrs_default() {
        let attrs = Attrs::default();
        assert!(attrs.attrs.is_empty());
    }

    #[test]
    fn test_attrs_from() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), json!("value1"));
        map.insert("key2".to_string(), json!(42));

        let attrs = Attrs::from(map.clone());
        assert_eq!(attrs.attrs, map);
    }

    #[test]
    fn test_index_access() {
        let mut attrs = Attrs::default();
        attrs["key1"] = json!("value1");
        attrs["key2"] = json!(42);
        println!("attrs key1: {:?}", attrs["key1"]);
        assert_eq!(attrs["key1"], json!("value1"));
        assert_eq!(attrs["key2"], json!(42));
    }

    #[test]
    fn test_index_mut_auto_create() {
        let mut attrs = Attrs::default();
        attrs["new_key"] = json!("new_value");
        println!("attrs new_key: {:?}", attrs["new_key"]);
        assert_eq!(attrs["new_key"], json!("new_value"));
    }

    #[test]
    fn test_get_value() {
        let mut attrs = Attrs::default();
        attrs["string"] = json!("test");
        attrs["number"] = json!(42);
        attrs["boolean"] = json!(true);

        assert_eq!(
            attrs.get_value::<String>("string"),
            Some("test".to_string())
        );
        assert_eq!(attrs.get_value::<i32>("number"), Some(42));
        assert_eq!(attrs.get_value::<bool>("boolean"), Some(true));
        assert_eq!(attrs.get_value::<String>("nonexistent"), None);
    }

    #[test]
    fn test_update() {
        let mut attrs = Attrs::default();
        attrs["key1"] = json!("value1");

        let mut new_values = HashMap::new();
        new_values.insert("key2".to_string(), json!("value2"));
        new_values.insert("key1".to_string(), json!("updated_value"));

        let updated = attrs.update(new_values);

        assert_eq!(updated["key1"], json!("updated_value"));
        assert_eq!(updated["key2"], json!("value2"));
    }

    #[test]
    fn test_deref() {
        let mut attrs = Attrs::default();
        attrs["key1"] = json!("value1");

        // Test Deref
        assert_eq!(attrs.get("key1"), Some(&json!("value1")));

        // Test DerefMut
        attrs.insert("key2".to_string(), json!("value2"));
        assert_eq!(attrs["key2"], json!("value2"));
    }

    #[test]
    #[should_panic(expected = "Key not found")]
    fn test_index_panic() {
        let attrs = Attrs::default();
        let _ = attrs["nonexistent"];
    }

    #[test]
    fn test_get_safe() {
        let mut attrs = Attrs::default();
        attrs["key1"] = json!("value1");

        assert_eq!(attrs.get_safe("key1"), Some(&json!("value1")));
        assert_eq!(attrs.get_safe("nonexistent"), None);
    }
}
