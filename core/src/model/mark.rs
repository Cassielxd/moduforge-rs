use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::attrs::Attrs;
/**
 * 基础标记实现 例如颜色 背景色 批注
 * @property type 标记类型
 * @property attrs 标记属性
 * @author string<348040933@qq.com>
 */
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Decode, Encode)]
pub struct Mark {
  #[serde(rename = "t")]
  pub r#type: String,
  #[serde(rename = "a")]
  #[bincode(with_serde)]
  pub attrs: Attrs,
}

impl Mark {
  pub fn set_from(marks: Option<Vec<Mark>>) -> Vec<Mark> {
    marks.unwrap_or_default()
  }
}
