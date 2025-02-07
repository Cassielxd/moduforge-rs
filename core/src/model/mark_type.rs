use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use super::mark::Mark;
use super::schema::{compute_attrs, Attribute, AttributeSpec, Schema};
use im::HashMap as ImHashMap;
#[derive(Clone, PartialEq, Debug, Eq)]
pub struct MarkType {
    pub name: String,
    pub rank: usize,
    pub schema: Option<Arc<Schema>>,
    pub spec: MarkSpec,
    pub attrs: HashMap<String, Attribute>,
    pub excluded: Option<Vec<MarkType>>,
}

impl MarkType {
    pub(crate) fn compile(marks: HashMap<String, MarkSpec>) -> HashMap<String, MarkType> {
        let mut result = HashMap::new();
        let mut rank = 0;

        for (name, spec) in marks {
            result.insert(
                name.clone(),
                MarkType::new(name.clone(), rank, None, spec.clone()),
            );
            rank += 1;
        }

        result
    }

    fn new(name: String, rank: usize, schema: Option<Arc<Schema>>, spec: MarkSpec) -> Self {
        let attrs = spec.attrs.as_ref().map_or_else(HashMap::new, |attrs| {
            attrs
                .iter()
                .map(|(name, spec)| (name.clone(), Attribute::new(&name, &name, spec.clone())))
                .collect()
        });

        MarkType {
            name,
            rank,
            schema,
            spec,
            attrs,
            excluded: None,
        }
    }

    fn create(&self, attrs: Option<&HashMap<String, String>>) -> Mark {
        Mark {
            r#type: self.name.clone(),
            attrs: self.compute_attrs(attrs),
        }
    }
    fn compute_attrs(
        &self,
        attrs: Option<&HashMap<String, String>>,
    ) -> ImHashMap<String, String> {
        match attrs {
            Some(attr) => compute_attrs(&self.attrs, Some(&attr)),
            None => compute_attrs(&self.attrs, None),
        }
    }

    // 其他方法...
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct MarkSpec {
    pub attrs: Option<HashMap<String, AttributeSpec>>,
    pub excludes: Option<String>,
    pub group: Option<String>,
    pub spanning: Option<bool>,
    pub desc: Option<String>,
}
