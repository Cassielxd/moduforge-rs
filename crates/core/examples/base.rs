use std::sync::Arc;

use mf_core::{
    middleware::Middleware, EditorOptionsBuilder, Extension, Extensions,
    ForgeActorRuntime, ForgeResult, RuntimeOptions,
};
use mf_model::{imbl::HashMap, NodeId};
use mf_state::{
    error::StateResult,
    plugin::{Plugin, PluginMetadata, PluginSpec, PluginTrait},
    State, Transaction,
};
use mf_transform::node_step::AddNodeStep;

/**
<?xml version="1.0" encoding="UTF-8"?>
<schema top_node="doc"
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xsi:noNamespaceSchemaLocation="moduforge-schema.xsd">
  <nodes>
    <!-- 工程项目 -->
    <node name="doc"  desc="工程项目" content="dwgc+" >
      <attrs>
        <attr name="title" default="工程项目"/>
        <attr name="price" default="0"/>
        <attr name="totalPrice" default="0"/>
      </attrs>
    </node>
    <node name="dwgc"  desc="单位项目" >
      <attrs>
        <attr name="name" default="单位项目"/>
        <attr name="price" default="0"/>
        <attr name="totalPrice" default="0"/>
      </attrs>
    </node>
  </nodes>
</schema>
 */

//定义插件工程项目 汇总 根据子节点单位项目 统计汇总
#[derive(Debug)]
struct APlugin;
#[async_trait::async_trait]
impl PluginTrait for APlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "a_plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "工程项目 汇总".to_string(),
            author: "a_plugin".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec![],
        }
    }
    async fn append_transaction(
        &self,
        trs: &[Arc<Transaction>],
        _: &Arc<State>,
        new_state: &Arc<State>,
    ) -> StateResult<Option<Transaction>> {
        println!("APlugin: append_transaction 被调用，事务数量: {}", trs.len());
        // 获取子单位工程 并汇总 前提 单位项目 计算完成之后
        let doc = new_state.doc();
        let mut price = 0;
        let mut total_price = 0;
        let mut new_tr = new_state.tr();

        for tr in trs {
            if let Some(dwgc_keys) = tr.get_meta::<Vec<NodeId>>("dwgcKeys") {
                for id in dwgc_keys {
                    if let Some(node) = doc.get_node(&id) {
                        let price1 = node
                            .attrs
                            .get_value::<i64>("price")
                            .unwrap_or_default();
                        price += price1;
                        let total_price1 = node
                            .attrs
                            .get_value::<i64>("totalPrice")
                            .unwrap_or_default();
                        total_price += total_price1;
                    }
                }
            }
        }
        let mut map = HashMap::new();
        map.insert("price".to_string(), price.into());
        map.insert("totalPrice".to_string(), total_price.into());
        new_tr.set_node_attribute(doc.root_id().clone(), map)?;
        println!("产生新的 汇总 事务");
        Ok(Some(new_tr))
    }
}

// 新增单位项目  之后的计算
#[derive(Debug)]
struct BPlugin;

#[async_trait::async_trait]
impl PluginTrait for BPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "b_plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "新增单位项目  之后的计算".to_string(),
            author: "b_plugin".to_string(),
            dependencies: vec![],
            conflicts: vec![],
            state_fields: vec![],
            tags: vec![],
        }
    }
    async fn append_transaction(
        &self,
        trs: &[Arc<Transaction>],
        _old_state: &Arc<State>,
        new_state: &Arc<State>,
    ) -> StateResult<Option<Transaction>> {
        println!("BPlugin: append_transaction 被调用，事务数量: {}", trs.len());
        // 如果 新增了 单位工程  需要计算并回填 金额相关数据
        let oss_pload =
            new_state.resource_manager().get::<OSSUpload>().unwrap();
        let sum = oss_pload.upload(1, 2);
        dbg!("计算结果{}", sum);
        let mut new_tr = new_state.tr();
        for tr in trs {
            for step in &tr.steps {
                if step.name() == "add_node_step" {
                    let node_step = step.downcast_ref::<AddNodeStep>().unwrap();
                    let add_node = &node_step.nodes[0].0;
                    if add_node.r#type == "dwgc" {
                        let id = add_node.id.clone();
                        let mut map = HashMap::new();
                        map.insert("price".to_string(), 100.into());
                        map.insert("totalPrice".to_string(), 1000.into());
                        new_tr.set_node_attribute(id.clone(), map)?;
                        new_tr.set_meta("dwgcKeys", vec![id]);
                    }
                }
            }
        }
        if new_tr.doc_changed() {
            dbg!("产生新的 单位工程 事务");
            return Ok(Some(new_tr));
        }
        Ok(None)
    }
}

struct LogMiddleware;

#[async_trait::async_trait]
impl Middleware for LogMiddleware {
    fn name(&self) -> String {
        "日志中间件".to_string()
    }

    /// 在事务到达核心分发之前处理事务
    async fn before_dispatch(
        &self,
        _transaction: &mut Transaction,
    ) -> ForgeResult<()> {
        println!("我是日志前置处理器");
        Ok(())
    }

    /// 在核心分发之后处理结果
    /// 返回一个可能包含需要额外处理的事务的 MiddlewareResult
    async fn after_dispatch(
        &self,
        _state: Option<Arc<State>>,
        transactions: &[Arc<Transaction>],
    ) -> ForgeResult<Option<Transaction>> {
        println!(
            "我是日志后置处理器，总共产生了 {:?} 个事务",
            transactions.len()
        );
        Ok(None)
    }
}

fn get_ops() -> RuntimeOptions {
    //添加默认插件
    EditorOptionsBuilder::new()
        .add_middleware(LogMiddleware)
        // 从XML schema文件加载节点定义
        .add_extension({
            let mut ext = Extension::new();
            ext.add_plugin(Arc::new(Plugin::new(PluginSpec {
                state_field: None,
                tr: Arc::new(APlugin),
            })));
            ext.add_plugin(Arc::new(Plugin::new(PluginSpec {
                state_field: None,
                tr: Arc::new(BPlugin),
            })));

            // 添加一个全局的资源管理器
            ext.add_op_fn(Arc::new(|op_state| {
                op_state.put(OSSUpload);
                Ok(())
            }));
            Extensions::E(ext)
        })
        .build()
}

#[derive(Clone)]
struct OSSUpload;
impl OSSUpload {
    pub fn upload(
        &self,
        a: i32,
        b: i32,
    ) -> i32 {
        a + b
    }
}

#[tokio::main]
async fn main() -> ForgeResult<()> {
    println!("开始创建编辑器...");
    let mut editor = ForgeActorRuntime::create(get_ops()).await?;
    println!("编辑器创建成功");

    let doc = editor.get_state().await?.doc();
    println!("获取初始文档状态:");
    dbg!(doc.clone());

    let state = editor.get_state().await?;
    let mut tr: Transaction = Transaction::new(&state);
    let schema = &tr.schema;
    let dw_node =
        schema.nodes["dwgc"].create_and_fill(None, None, vec![], None, schema);
    tr.add_node(doc.root_id().clone(), vec![dw_node])?;

    println!("准备分发事务...");
    editor.dispatch(tr).await?;
    println!("事务分发完成");

    // 运行编辑器
    println!("获取最终文档状态:");
    dbg!(editor.get_state().await?.doc());
    Ok(())
}
