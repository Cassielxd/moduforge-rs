use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::{attrs::Attrs, node_factory::NodeFactory};

use super::mark::Mark;
use super::schema::{Attribute, AttributeSpec, compute_attrs};
#[derive(Clone, PartialEq, Debug, Eq)]
pub struct MarkDefinition {
    pub name: String,
    pub rank: usize,
    pub spec: MarkSpec,
    pub attrs: HashMap<String, Attribute>,
    pub excluded: Option<Vec<MarkDefinition>>,
}

impl MarkDefinition {
    pub(crate) fn compile(
        marks: HashMap<String, MarkSpec>
    ) -> HashMap<String, MarkDefinition> {
        let mut result = HashMap::new();

        for (rank, (name, spec)) in marks.into_iter().enumerate() {
            result.insert(
                name.clone(),
                MarkDefinition::new(name.clone(), rank, spec.clone()),
            );
        }

        result
    }

    fn new(
        name: String,
        rank: usize,
        spec: MarkSpec,
    ) -> Self {
        let attrs = spec.attrs.as_ref().map_or_else(HashMap::new, |attrs| {
            attrs
                .iter()
                .map(|(name, spec)| {
                    (name.clone(), Attribute::new(spec.clone()))
                })
                .collect()
        });

        MarkDefinition { name, rank, spec, attrs, excluded: None }
    }

    pub fn create(
        &self,
        attrs: Option<&HashMap<String, Value>>,
    ) -> Mark {
        NodeFactory::instantiate_mark(self, attrs)
    }
    pub fn compute_attrs(
        &self,
        attrs: Option<&HashMap<String, Value>>,
    ) -> Attrs {
        match attrs {
            Some(attr) => compute_attrs(&self.attrs, Some(attr)),
            None => compute_attrs(&self.attrs, None),
        }
    }

    // 其他方法...
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Default)]
pub struct MarkSpec {
    pub attrs: Option<HashMap<String, AttributeSpec>>,
    pub excludes: Option<String>,
    pub group: Option<String>,
    pub spanning: Option<bool>,
    pub desc: Option<String>,
}


