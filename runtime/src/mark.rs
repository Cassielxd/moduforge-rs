use moduforge_core::model::mark_type::MarkSpec;

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Mark {
    pub name: String,
    pub r#type: MarkSpec,
}

impl Mark {
    pub fn new(name: &str, spec: MarkSpec) -> Mark {
        Mark {
            name: name.to_string(),
            r#type: spec,
        }
    }
}
