use std::time::{SystemTime, UNIX_EPOCH};
use mf_model::mark::Mark;
use mf_model::tree::Tree;
use mf_state::transaction::Transaction;
// 新版本映射模块已经通过 crate::mapping 提供所有必要的API
use crate::AwarenessRef;
use serde_json::Value as JsonValue;
use yrs::{Map, ReadTxn, Transact};
use yrs::{
    types::{array::ArrayRef, map::MapRef, Value},
    Array, ArrayPrelim, MapPrelim, TransactionMut, WriteTxn,
};

use crate::ClientResult;
use mf_model::{node::Node, attrs::Attrs, types::NodeId};
use std::sync::Arc;
use std::collections::HashMap;

/// 获取当前时间戳（毫秒）
pub struct Utils;
impl Utils {
    pub fn get_unix_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
    /// 将 Yrs 文档转换为 ModuForge Tree
    /// 这是从协作状态重建文档树的关键方法
    pub fn apply_yrs_to_tree(doc: &yrs::Doc) -> ClientResult<Tree> {
        use mf_model::types::NodeId;
        use std::collections::HashMap;

        let root_id = Utils::get_root_id_from_yrs_doc(doc)?;
        let txn = doc.transact();
        let nodes_map = txn
            .get_map("nodes")
            .ok_or(anyhow::anyhow!("Yrs 文档中没有找到 nodes 映射"))?;
        let mut tree_nodes = HashMap::new();
        let mut parent_map = HashMap::new();

        Utils::build_tree_nodes_from_yrs(
            &root_id,
            &nodes_map,
            &txn,
            &mut tree_nodes,
            &mut parent_map,
            None,
        )?;

        let root_node = tree_nodes
            .get(&NodeId::from(root_id))
            .ok_or(anyhow::anyhow!("根节点不存在"))?
            .as_ref()
            .clone();
        let root_enum =
            Utils::build_node_enum_from_map(&root_node, &tree_nodes);
        Ok(Tree::from(root_enum))
    }
    /// 初始化树
    /// 将树同步到 Yrs 文档
    pub async fn apply_tree_to_yrs(
        awareness_ref: AwarenessRef,
        tree: &Tree,
    ) -> ClientResult<()> {
        let mut awareness = awareness_ref.write().await;
        let doc = awareness.doc_mut();
        let mut txn =
            doc.transact_mut_with(doc.client_id().clone().to_string());

        // 清空现有数据（如果有的话）
        let nodes_map = txn.get_or_insert_map("nodes");
        nodes_map.clear(&mut txn);

        // 同步 Tree 中的所有节点到 Yrs 文档
        Utils::sync_tree_to_yrs(tree, &mut txn)?;

        // 添加根节点ID到 meta 区域
        let meta_map = txn.get_or_insert_map("meta");
        meta_map.insert(
            &mut txn,
            "root_id",
            yrs::Any::String(tree.root_id.to_string().into()),
        );

        // 提交事务
        txn.commit();

        tracing::info!(
            "成功初始化树，包含 {} 个节点，根节点ID: {}",
            tree.nodes.iter().map(|shard| shard.len()).sum::<usize>(),
            tree.root_id
        );
        Ok(())
    }

    /// 将树同步到 Yrs 文档
    pub fn sync_tree_to_yrs(
        tree: &Tree,
        txn: &mut yrs::TransactionMut,
    ) -> ClientResult<()> {
        use mf_transform::node_step::AddNodeStep;

        // 获取根节点的所有子树
        if let Some(root_tree) = tree.all_children(&tree.root_id, None) {
            // 创建一个 AddNodeStep 来添加整个子树
            // 注意：root_tree 已经包含了根节点，不需要重复添加
            let add_step = AddNodeStep {
                parent_id: tree.root_id.clone(),
                nodes: vec![root_tree],
            };

            // 使用新版本的转换器API
            let context = crate::mapping::create_context(
                "tree_sync_client".to_string(),
                "tree_sync_user".to_string(),
            );

            if let Err(e) =
                crate::mapping::convert_step(&add_step, txn, &context)
            {
                tracing::error!("🔄 同步树节点到 Yrs 失败: {}", e);
                return Err(anyhow::anyhow!(format!(
                    "Failed to sync tree: {}",
                    e
                )));
            }
        }

        Ok(())
    }
    /// 将事务应用到 Yrs 文档
    pub async fn apply_transaction_to_yrs(
        awareness_ref: AwarenessRef,
        transaction: &Transaction,
    ) -> ClientResult<()> {
        // 使用异步锁获取房间信息

        let mut awareness = awareness_ref.write().await;
        let doc = awareness.doc_mut();
        let mut txn =
            doc.transact_mut_with(doc.client_id().clone().to_string());
        // 使用新版本的转换API应用所有事务中的步骤
        let context = crate::mapping::create_context(
            "transaction_client".to_string(),
            "transaction_user".to_string(),
        );

        let steps = &transaction.steps;
        for step in steps {
            if let Err(e) =
                crate::mapping::convert_step(step.as_ref(), &mut txn, &context)
            {
                tracing::error!("🔄 应用步骤到 Yrs 事务失败: {}", e);
            }
        }
        // 统一提交所有更改
        txn.commit();
        tracing::debug!(
            "🔄 应用 {} 个步骤到文档: {}",
            transaction.steps.len(),
            doc.client_id()
        );

        Ok(())
    }

    /// 将事务应用到 Yrs 文档
    pub async fn apply_transactions_to_yrs(
        awareness_ref: AwarenessRef,
        transactions: &[Transaction],
    ) -> ClientResult<()> {
        // 使用异步锁获取房间信息

        let mut awareness = awareness_ref.write().await;
        let doc = awareness.doc_mut();
        let mut txn =
            doc.transact_mut_with(doc.client_id().clone().to_string());

        // 使用新版本的转换API应用所有事务中的步骤
        let context = crate::mapping::create_context(
            "bulk_transaction_client".to_string(),
            "bulk_transaction_user".to_string(),
        );

        for tr in transactions {
            let steps = &tr.steps;
            for step in steps {
                if let Err(e) = crate::mapping::convert_step(
                    step.as_ref(),
                    &mut txn,
                    &context,
                ) {
                    tracing::error!("🔄 应用步骤到 Yrs 事务失败: {}", e);
                }
            }
        }
        // 统一提交所有更改
        txn.commit();
        tracing::debug!(
            "🔄 应用 {} 个事务到文档: {}",
            transactions.len(),
            doc.client_id()
        );

        Ok(())
    }

    // --- 转换器的辅助方法 ---
    /// 将 JSON 值转换为 Yrs 的 Any 类型
    pub fn json_value_to_yrs_any(value: &JsonValue) -> yrs::Any {
        match value {
            JsonValue::Null => yrs::Any::Null,
            JsonValue::Bool(b) => yrs::Any::Bool(*b),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_i64() {
                    yrs::Any::BigInt(i)
                } else if let Some(f) = n.as_f64() {
                    yrs::Any::Number(f)
                } else {
                    yrs::Any::Null
                }
            },
            JsonValue::String(s) => yrs::Any::String(s.clone().into()),
            JsonValue::Array(arr) => {
                let yrs_array: Vec<yrs::Any> =
                    arr.iter().map(Utils::json_value_to_yrs_any).collect();
                yrs::Any::Array(yrs_array.into())
            },
            JsonValue::Object(obj) => {
                let yrs_map: std::collections::HashMap<String, yrs::Any> = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), Utils::json_value_to_yrs_any(v)))
                    .collect();
                yrs::Any::Map(yrs_map.into())
            },
        }
    }

    /// 将标记添加到 Yrs 数组中
    pub fn add_mark_to_array(
        marks_array: &ArrayRef,
        txn: &mut TransactionMut,
        mark: &Mark,
    ) {
        let mark_map = MapPrelim::<yrs::Any>::from([
            ("type".to_string(), yrs::Any::String(mark.r#type.clone().into())),
            ("attrs".to_string(), {
                let attrs_map: std::collections::HashMap<String, yrs::Any> =
                    mark.attrs
                        .iter()
                        .map(|(k, v)| {
                            (k.clone(), Utils::json_value_to_yrs_any(v))
                        })
                        .collect();
                yrs::Any::Map(attrs_map.into())
            }),
        ]);
        marks_array.push_back(txn, mark_map);
    }

    /// 获取或创建节点数据映射
    pub fn get_or_create_node_data_map(
        nodes_map: &MapRef,
        txn: &mut TransactionMut,
        node_id: &str,
    ) -> MapRef {
        if let Some(Value::YMap(map)) = nodes_map.get(txn, node_id) {
            map
        } else {
            nodes_map.insert(
                txn,
                node_id.to_string(),
                MapPrelim::<yrs::Any>::new(),
            )
        }
    }

    /// 获取或创建节点属性映射
    pub fn get_or_create_node_attrs_map(
        node_data_map: &MapRef,
        txn: &mut TransactionMut,
    ) -> MapRef {
        if let Some(Value::YMap(map)) = node_data_map.get(txn, "attrs") {
            map
        } else {
            node_data_map.insert(txn, "attrs", MapPrelim::<yrs::Any>::new())
        }
    }

    /// 获取或创建标记数组
    pub fn get_or_create_content_array(
        node_data_map: &MapRef,
        txn: &mut TransactionMut,
    ) -> ArrayRef {
        if let Some(Value::YArray(array)) = node_data_map.get(txn, "content") {
            array
        } else {
            node_data_map.insert(
                txn,
                "content",
                ArrayPrelim::from(Vec::<yrs::Any>::new()),
            )
        }
    }

    /// 将 Yrs 的 Any 类型转换为 JSON 值
    pub fn yrs_any_to_json_value(value: &yrs::Any) -> Option<JsonValue> {
        match value {
            yrs::Any::Null => Some(JsonValue::Null),
            yrs::Any::Bool(b) => Some(JsonValue::Bool(*b)),
            yrs::Any::Number(n) => {
                Some(JsonValue::Number(serde_json::Number::from_f64(*n)?))
            },
            yrs::Any::BigInt(i) => {
                Some(JsonValue::Number(serde_json::Number::from(*i)))
            },
            yrs::Any::String(s) => Some(JsonValue::String(s.to_string())),
            yrs::Any::Array(arr) => {
                let json_array: Vec<JsonValue> = arr
                    .iter()
                    .filter_map(Utils::yrs_any_to_json_value)
                    .collect();
                Some(JsonValue::Array(json_array))
            },
            yrs::Any::Map(map) => {
                let json_map: std::collections::HashMap<String, JsonValue> =
                    map.iter()
                        .filter_map(|(k, v)| {
                            Utils::yrs_any_to_json_value(v)
                                .map(|json_v| (k.to_string(), json_v))
                        })
                        .collect();
                Some(JsonValue::Object(serde_json::Map::from_iter(json_map)))
            },
            _ => None, // 处理其他类型，如 YText, YMap 等
        }
    }

    /// 获取或创建标记数组
    pub fn get_or_create_marks_array(
        node_data_map: &MapRef,
        txn: &mut TransactionMut,
    ) -> ArrayRef {
        if let Some(Value::YArray(array)) = node_data_map.get(txn, "marks") {
            array
        } else {
            node_data_map.insert(
                txn,
                "marks",
                ArrayPrelim::from(Vec::<yrs::Any>::new()),
            )
        }
    }

    /// 从 Yrs 文档中获取根节点ID
    pub fn get_root_id_from_yrs_doc(doc: &yrs::Doc) -> ClientResult<String> {
        let txn = doc.transact();
        // 优先从 meta 区域读取
        if let Some(meta_map) = txn.get_map("meta") {
            if let Some(yrs::types::Value::Any(any)) =
                meta_map.get(&txn, "root_id")
            {
                return Ok(any.to_string());
            }
        }
        // fallback: 兼容老数据，取 nodes_map 第一个节点
        let nodes_map = txn
            .get_map("nodes")
            .ok_or(anyhow::anyhow!("Yrs 文档中没有找到 nodes 映射"))?;
        if let Some((key, _)) = nodes_map.iter(&txn).next() {
            return Ok(key.to_string());
        }
        Err(anyhow::anyhow!("Yrs 文档中没有找到根节点"))
    }

    /// 从 Yrs 文档的 nodes_map 递归构建所有节点和 parent_map
    pub fn build_tree_nodes_from_yrs(
        node_id: &str,
        nodes_map: &yrs::types::map::MapRef,
        txn: &yrs::Transaction,
        tree_nodes: &mut HashMap<NodeId, Arc<Node>>,
        parent_map: &mut HashMap<NodeId, NodeId>,
        parent_id: Option<&NodeId>,
    ) -> ClientResult<()> {
        let node_data = nodes_map.get(txn, node_id);
        if node_data.is_none() {
            return Err(anyhow::anyhow!(
                "节点 {} 在 Yrs 文档中不存在",
                node_id
            ));
        }
        let node_data = node_data.unwrap();
        if let yrs::types::Value::YMap(node_map) = node_data {
            // 提取节点类型
            let node_type = node_map
                .get(txn, "type")
                .and_then(|v| match v {
                    yrs::types::Value::Any(any) => Some(any.to_string()),
                    _ => None,
                })
                .unwrap_or_else(|| "unknown".to_string());

            // 提取属性
            let mut attrs = Attrs::default();
            if let Some(yrs::types::Value::YMap(attrs_yrs_map)) = node_map.get(txn, "attrs") {
                for (key, value) in attrs_yrs_map.iter(txn) {
                    if let yrs::types::Value::Any(any_value) = value {
                        if let Some(json_value) =
                            Utils::yrs_any_to_json_value(&any_value)
                        {
                            attrs.insert(key.to_string(), json_value);
                        }
                    }
                }
            }

            // 提取内容（子节点ID列表）
            let mut content = imbl::Vector::new();
            if let Some(yrs::types::Value::YArray(content_yrs_array)) = node_map.get(txn, "content") {
                for item in content_yrs_array.iter(txn) {
                    if let yrs::types::Value::Any(any) = item {
                        content.push_back(NodeId::from(any.to_string()));
                    }
                }
            }

            // 提取标记
            let mut marks = imbl::Vector::new();
            if let Some(yrs::types::Value::YArray(marks_yrs_array)) = node_map.get(txn, "marks") {
                for item in marks_yrs_array.iter(txn) {
                    if let yrs::types::Value::YMap(mark_map) = item {
                        let mark_type = mark_map
                            .get(txn, "type")
                            .and_then(|v| match v {
                                yrs::types::Value::Any(any) => {
                                    Some(any.to_string())
                                },
                                _ => None,
                            })
                            .unwrap_or_else(|| "unknown".to_string());

                        let mut mark_attrs = Attrs::default();
                        if let Some(yrs::types::Value::YMap(attrs_yrs_map)) =
                            mark_map.get(txn, "attrs")
                        {
                            for (key, value) in attrs_yrs_map.iter(txn)
                            {
                                if let yrs::types::Value::Any(
                                    any_value,
                                ) = value
                                {
                                    if let Some(json_value) =
                                        Utils::yrs_any_to_json_value(
                                            &any_value,
                                        )
                                    {
                                        mark_attrs.insert(
                                            key.to_string(),
                                            json_value,
                                        );
                                    }
                                }
                            }
                        }

                        marks.push_back(Mark {
                            r#type: mark_type,
                            attrs: mark_attrs,
                        });
                    }
                }
            }

            // 创建节点
            let content_vec: Vec<NodeId> =
                content.clone().into_iter().collect();
            let marks_vec: Vec<Mark> = marks.clone().into_iter().collect();
            let node =
                Node::new(node_id, node_type, attrs, content_vec, marks_vec);

            let node_id_typed = NodeId::from(node_id);
            tree_nodes.insert(node_id_typed.clone(), Arc::new(node));

            // 记录父子关系
            if let Some(parent) = parent_id {
                parent_map.insert(node_id_typed.clone(), parent.clone());
            }

            // 递归处理子节点
            for child_id in content {
                Utils::build_tree_nodes_from_yrs(
                    &child_id,
                    nodes_map,
                    txn,
                    tree_nodes,
                    parent_map,
                    Some(&node_id_typed),
                )?;
            }
        }
        Ok(())
    }

    /// 递归构建 NodeEnum
    pub fn build_node_enum_from_map(
        node: &Node,
        tree_nodes: &HashMap<NodeId, Arc<Node>>,
    ) -> mf_model::node_type::NodeEnum {
        let mut children = Vec::new();
        for child_id in &node.content {
            if let Some(child_node) = tree_nodes.get(child_id) {
                children.push(Utils::build_node_enum_from_map(
                    child_node, tree_nodes,
                ));
            }
        }
        mf_model::node_type::NodeEnum(node.clone(), children)
    }
}
