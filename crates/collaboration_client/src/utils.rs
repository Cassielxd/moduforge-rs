use std::time::{SystemTime, UNIX_EPOCH};
use mf_model::mark::Mark;
use mf_model::tree::Tree;
use mf_state::Transaction;
use yrs_warp::AwarenessRef;
use serde_json::Value as JsonValue;
use yrs::{Map, ReadTxn as _, Transact};
use yrs::{
    types::{array::ArrayRef, map::MapRef, Value},
    Array, ArrayPrelim, MapPrelim, TransactionMut, WriteTxn,
};

use crate::{mapping::Mapper, ClientResult};
/// 获取当前时间戳（毫秒）
pub fn get_unix_time() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis()
        as u64
}

/// 初始化树
/// 将树同步到 Yrs 文档
pub async fn init_tree(
    awareness_ref:AwarenessRef,
    tree: &Tree,
) -> ClientResult<()> {
    let mut awareness = awareness_ref.write().await;
    let doc = awareness.doc_mut();
    let mut txn = doc.transact_mut_with(doc.client_id().clone());

    // 清空现有数据（如果有的话）
    let nodes_map = txn.get_or_insert_map("nodes");
    nodes_map.clear(&mut txn);

    // 同步 Tree 中的所有节点到 Yrs 文档
    sync_tree_to_yrs(tree, &mut txn)?;
    // 提交事务
    txn.commit();

    tracing::info!(
            "成功初始化树，包含 {} 个节点",tree.nodes.len()
        );
    Ok(())
}

/// 将树同步到 Yrs 文档
pub fn sync_tree_to_yrs(
    tree: &Tree,
    txn: &mut yrs::TransactionMut,
) -> ClientResult<()> {
    use mf_transform::{step::Step, node_step::AddNodeStep};

    let registry = Mapper::global_registry();

    // 获取根节点的所有子树
    if let Some(root_tree) = tree.all_children(&tree.root_id, None) {
        // 创建一个 AddNodeStep 来添加整个子树
        let add_step = AddNodeStep {
            parent_id: tree.root_id.clone(),
            nodes: vec![root_tree],
        };

        // 使用现有的转换器应用步骤
        if let Some(converter) =
            registry.find_converter(&add_step as &dyn Step)
        {
            if let Err(e) =
                converter.apply_to_yrs_txn(&add_step as &dyn Step, txn)
            {
                tracing::error!("🔄 同步树节点到 Yrs 失败: {}", e);
                return Err(anyhow::anyhow!(
                    format!("Failed to sync tree: {}", e),
                ));
            }
        } else {
            tracing::error!(
                "🔄 同步树节点到 Yrs 失败: 没有找到 AddNodeStep 的转换器"
            );
            return Err(anyhow::anyhow!("No converter found for AddNodeStep"));
        }
    }

    Ok(())
}

/// 将事务应用到 Yrs 文档
pub async fn apply_transactions_to_yrs(
    awareness_ref:AwarenessRef,
    transactions: &[Transaction],
) -> ClientResult<()> {
    // 使用异步锁获取房间信息
 
        let mut awareness = awareness_ref.write().await;
        let doc = awareness.doc_mut();
        let mut txn = doc.transact_mut_with(doc.client_id().clone());

        // 使用全局注册表应用所有事务中的步骤
        let registry = Mapper::global_registry();

        for tr in transactions {
            let steps = &tr.steps;
            for step in steps {
                if let Some(converter) =
                    registry.find_converter(step.as_ref())
                {
                    if let Err(e) =
                        converter.apply_to_yrs_txn(step.as_ref(), &mut txn)
                    {
                        tracing::error!(
                            "🔄 应用步骤到 Yrs 事务失败: {}",
                            e
                        );
                    }
                } else {
                    let type_name =
                        std::any::type_name_of_val(step.as_ref());
                    tracing::warn!(
                        "🔄 应用步骤到 Yrs 事务失败: 没有找到步骤的转换器: {}",
                        type_name
                    );
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
                arr.iter().map(json_value_to_yrs_any).collect();
            yrs::Any::Array(yrs_array.into())
        },
        JsonValue::Object(obj) => {
            let yrs_map: std::collections::HashMap<String, yrs::Any> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_value_to_yrs_any(v)))
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
            let attrs_map: std::collections::HashMap<String, yrs::Any> = mark
                .attrs
                .iter()
                .map(|(k, v)| (k.clone(), json_value_to_yrs_any(v)))
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
        nodes_map.insert(txn, node_id.to_string(), MapPrelim::<yrs::Any>::new())
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
