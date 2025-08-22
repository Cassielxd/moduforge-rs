use serde::{Deserialize, Serialize};
use serde_json::Value;

/// XML Schema 根结构（基础版本）
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "schema")]
pub struct XmlSchema {
    #[serde(rename = "@top_node")]
    pub top_node: Option<String>,
    pub nodes: Option<XmlNodes>,
    pub marks: Option<XmlMarks>,
}

/// 支持引用的XML Schema 根结构（完整版本）
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename = "schema")]
pub struct XmlSchemaWithReferences {
    #[serde(rename = "@top_node")]
    pub top_node: Option<String>,
    pub imports: Option<XmlImports>,
    pub includes: Option<XmlIncludes>,
    pub global_attributes: Option<XmlGlobalAttributes>,
    pub nodes: Option<XmlNodes>,
    pub marks: Option<XmlMarks>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct XmlImports {
    #[serde(rename = "import")]
    pub imports: Vec<XmlImport>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct XmlIncludes {
    #[serde(rename = "include")]
    pub includes: Vec<XmlInclude>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct XmlImport {
    #[serde(rename = "@src")]
    pub src: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct XmlInclude {
    #[serde(rename = "@src")]
    pub src: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct XmlGlobalAttributes {
    #[serde(rename = "global_attribute")]
    pub global_attributes: Vec<XmlGlobalAttribute>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct XmlGlobalAttribute {
    #[serde(rename = "@types")]
    pub types: String,
    #[serde(rename = "attr")]
    pub attrs: Vec<XmlAttr>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct XmlNodes {
    #[serde(rename = "node")]
    pub nodes: Vec<XmlNode>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct XmlMarks {
    #[serde(rename = "mark")]
    pub marks: Vec<XmlMark>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct XmlNode {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@group")]
    pub group: Option<String>,
    #[serde(rename = "@desc")]
    pub desc: Option<String>,
    #[serde(rename = "@content")]
    pub content: Option<String>,
    #[serde(rename = "@marks")]
    pub marks: Option<String>,
    pub attrs: Option<XmlAttrs>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct XmlMark {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@group")]
    pub group: Option<String>,
    #[serde(rename = "@desc")]
    pub desc: Option<String>,
    #[serde(rename = "@excludes")]
    pub excludes: Option<String>,
    #[serde(
        rename = "@spanning",
        deserialize_with = "deserialize_optional_bool",
        default
    )]
    pub spanning: Option<bool>,
    pub attrs: Option<XmlAttrs>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct XmlAttrs {
    #[serde(rename = "attr")]
    pub attrs: Vec<XmlAttr>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct XmlAttr {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(
        rename = "@default",
        deserialize_with = "deserialize_optional_value",
        default
    )]
    pub default: Option<Value>,
}

// -------- 自定义反序列化器 --------

pub fn deserialize_optional_value<'de, D>(
    deserializer: D
) -> Result<Option<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            let value = match s.as_str() {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                _ => {
                    if let Ok(num) = s.parse::<i64>() {
                        Value::Number(serde_json::Number::from(num))
                    } else if let Ok(num) = s.parse::<f64>() {
                        if let Some(json_num) =
                            serde_json::Number::from_f64(num)
                        {
                            Value::Number(json_num)
                        } else {
                            Value::String(s)
                        }
                    } else {
                        match serde_json::from_str::<Value>(&s) {
                            Ok(parsed_value) => parsed_value,
                            Err(_) => Value::String(s),
                        }
                    }
                },
            };
            Ok(Some(value))
        },
        None => Ok(None),
    }
}

pub fn deserialize_optional_bool<'de, D>(
    deserializer: D
) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            if s == "true" {
                Ok(Some(true))
            } else if s == "false" {
                Ok(Some(false))
            } else {
                Ok(Some(false))
            }
        },
        None => Ok(Some(false)),
    }
}
