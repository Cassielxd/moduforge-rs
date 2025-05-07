use std::ops::{Deref, DerefMut};

use im::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
//pub type Attrs = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Attrs {
    pub attrs: HashMap<String, Value>,
}

impl Default for Attrs {
    fn default() -> Self {
        Self { attrs: HashMap::new() }
    }
}

impl Attrs {
    pub fn from(new_values: HashMap<String, Value>) -> Self {
        Self { attrs: new_values }
    }
    pub fn get_value<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.attrs.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    pub fn update(&self,new_values: HashMap<String, Value>,)->Self{
        let mut attrs = self.attrs.clone();
        for (key, value) in new_values {
            attrs.insert(key, value);
        }   
        Attrs { attrs }
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