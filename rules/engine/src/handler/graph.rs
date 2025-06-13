use crate::handler::custom_node_adapter::{CustomNodeAdapter, CustomNodeRequest};
use crate::handler::decision::DecisionHandler;
use crate::handler::expression::ExpressionHandler;
use crate::handler::function::function::{Function, FunctionConfig};
use crate::handler::function::module::console::ConsoleListener;
use crate::handler::function::module::custom::ModuforgeListener;
use crate::handler::function::module::zen::ZenListener;
use crate::handler::function::FunctionHandler;
use crate::handler::function_v1;
use crate::handler::function_v1::runtime::create_runtime;
use crate::handler::node::{NodeRequest, PartialTraceError};
use crate::handler::table::zen::DecisionTableHandler;
use crate::handler::traversal::{GraphWalker, StableDiDecisionGraph};
use crate::loader::DecisionLoader;
use crate::model::{DecisionContent, DecisionNodeKind, FunctionNodeContent};
use crate::util::validator_cache::ValidatorCache;
use crate::{EvaluationError, NodeError};
use ahash::{HashMap, HashMapExt};
use anyhow::anyhow;
use petgraph::algo::is_cyclic_directed;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use moduforge_rules_expression::variable::Variable;

/// 决策图结构体
/// 用于表示和管理决策图，包含图的构建、验证和评估功能
pub struct DecisionGraph<
    L: DecisionLoader + 'static,
    A: CustomNodeAdapter + 'static,
> {
    /// 初始图结构，用于重置
    initial_graph: StableDiDecisionGraph,
    /// 当前图结构，可能经过修改
    graph: StableDiDecisionGraph,
    /// 自定义节点适配器
    adapter: Arc<A>,
    /// 决策加载器
    loader: Arc<L>,
    /// 是否启用跟踪
    trace: bool,
    /// 最大深度限制
    max_depth: u8,
    /// 当前迭代次数
    iteration: u8,
    /// 运行时函数
    runtime: Option<Rc<Function>>,
    /// 验证器缓存
    validator_cache: ValidatorCache,
}

/// 决策图配置结构体
/// 用于初始化决策图的配置参数
pub struct DecisionGraphConfig<
    L: DecisionLoader + 'static,
    A: CustomNodeAdapter + 'static,
> {
    /// 决策加载器
    pub loader: Arc<L>,
    /// 自定义节点适配器
    pub adapter: Arc<A>,
    /// 决策内容
    pub content: Arc<DecisionContent>,
    /// 是否启用跟踪
    pub trace: bool,
    /// 迭代次数
    pub iteration: u8,
    /// 最大深度限制
    pub max_depth: u8,
    /// 验证器缓存
    pub validator_cache: Option<ValidatorCache>,
}

impl<L: DecisionLoader + 'static, A: CustomNodeAdapter + 'static>
    DecisionGraph<L, A>
{
    /// 创建新的决策图实例
    ///
    /// # 参数
    /// * `config` - 决策图配置
    ///
    /// # 返回
    /// * `Result<Self, DecisionGraphValidationError>` - 成功返回决策图实例，失败返回验证错误
    pub fn try_new(
        config: DecisionGraphConfig<L, A>
    ) -> Result<Self, DecisionGraphValidationError> {
        let content = config.content;
        let mut graph = StableDiDecisionGraph::new();
        let mut index_map = HashMap::new();

        // 添加所有节点到图中
        for node in &content.nodes {
            let node_id = node.id.clone();
            let node_index = graph.add_node(node.clone());
            index_map.insert(node_id, node_index);
        }

        // 添加所有边到图中
        for (_, edge) in content.edges.iter().enumerate() {
            let source_index =
                index_map.get(&edge.source_id).ok_or_else(|| {
                    DecisionGraphValidationError::MissingNode(
                        edge.source_id.to_string(),
                    )
                })?;

            let target_index =
                index_map.get(&edge.target_id).ok_or_else(|| {
                    DecisionGraphValidationError::MissingNode(
                        edge.target_id.to_string(),
                    )
                })?;

            graph.add_edge(
                source_index.clone(),
                target_index.clone(),
                edge.clone(),
            );
        }

        Ok(Self {
            initial_graph: graph.clone(),
            graph,
            iteration: config.iteration,
            trace: config.trace,
            loader: config.loader,
            adapter: config.adapter,
            max_depth: config.max_depth,
            validator_cache: config.validator_cache.unwrap_or_default(),
            runtime: None,
        })
    }

    /// 设置运行时函数
    pub(crate) fn with_function(
        mut self,
        runtime: Option<Rc<Function>>,
    ) -> Self {
        self.runtime = runtime;
        self
    }

    /// 重置图到初始状态
    pub(crate) fn reset_graph(&mut self) {
        self.graph = self.initial_graph.clone();
    }

    /// 获取或创建运行时函数
    async fn get_or_insert_function(&mut self) -> anyhow::Result<Rc<Function>> {
        if let Some(function) = &self.runtime {
            return Ok(function.clone());
        }

        // 创建新的运行时函数
        let function = Function::create(FunctionConfig {
            listeners: Some(vec![
                Box::new(ModuforgeListener {}),
                Box::new(ConsoleListener),
                Box::new(ZenListener {
                    loader: self.loader.clone(),
                    adapter: self.adapter.clone(),
                }),
            ]),
        })
        .await
        .map_err(|err| anyhow!(err.to_string()))?;
        let rc_function = Rc::new(function);
        self.runtime.replace(rc_function.clone());

        Ok(rc_function)
    }

    /// 验证决策图的有效性
    ///
    /// # 验证内容
    /// 1. 检查输入节点数量是否为1
    /// 2. 检查是否存在循环依赖
    pub fn validate(&self) -> Result<(), DecisionGraphValidationError> {
        let input_count = self.input_node_count();
        if input_count != 1 {
            return Err(DecisionGraphValidationError::InvalidInputCount(
                input_count as u32,
            ));
        }

        if is_cyclic_directed(&self.graph) {
            return Err(DecisionGraphValidationError::CyclicGraph);
        }

        Ok(())
    }

    /// 计算输入节点的数量
    fn input_node_count(&self) -> usize {
        self.graph
            .node_weights()
            .filter(|weight| {
                matches!(
                    weight.kind,
                    DecisionNodeKind::InputNode { content: _ }
                )
            })
            .count()
    }

    /// 评估决策图
    ///
    /// # 参数
    /// * `context` - 输入上下文变量
    ///
    /// # 返回
    /// * `Result<DecisionGraphResponse, NodeError>` - 评估结果或错误
    pub async fn evaluate(
        &mut self,
        context: Variable,
    ) -> Result<DecisionGraphResponse, NodeError> {
        let root_start = Instant::now();

        // 验证图的有效性
        self.validate().map_err(|e| NodeError {
            node_id: "".to_string(),
            source: anyhow!(e),
            trace: None,
        })?;

        // 检查是否超过最大深度限制
        if self.iteration >= self.max_depth {
            return Err(NodeError {
                node_id: "".to_string(),
                source: anyhow!(EvaluationError::DepthLimitExceeded),
                trace: None,
            });
        }

        // 创建图遍历器并开始遍历
        let mut walker = GraphWalker::new(&self.graph);
        let mut node_traces = self.trace.then(|| HashMap::default());

        // 遍历图中的所有节点
        while let Some(nid) = walker.next(
            &mut self.graph,
            self.trace.then_some(|mut trace: DecisionGraphTrace| {
                if let Some(nt) = &mut node_traces {
                    trace.order = nt.len() as u32;
                    nt.insert(trace.id.clone(), trace);
                };
            }),
        ) {
            // 如果节点已有数据，跳过处理
            if let Some(_) = walker.get_node_data(nid) {
                continue;
            }

            let node = (&self.graph[nid]).clone();
            let start = Instant::now();

            // 定义跟踪宏
            macro_rules! trace {
                ({ $($field:ident: $value:expr),* $(,)? }) => {
                    if let Some(nt) = &mut node_traces {
                        nt.insert(
                            node.id.clone(),
                            DecisionGraphTrace {
                                name: node.name.clone(),
                                id: node.id.clone(),
                                performance: Some(format!("{:.1?}", start.elapsed())),
                                order: nt.len() as u32,
                                $($field: $value,)*
                            }
                        );
                    }
                };
            }

            // 根据节点类型处理
            match &node.kind {
                // 处理输入节点
                DecisionNodeKind::InputNode { content } => {
                    trace!({
                        input: Variable::Null,
                        output: context.clone(),
                        trace_data: None,
                    });

                    // 验证输入数据
                    if let Some(json_schema) = content
                        .schema
                        .as_ref()
                        .map(|s| serde_json::from_str::<Value>(&s).ok())
                        .flatten()
                    {
                        let validator_key =
                            create_validator_cache_key(&json_schema);
                        let validator = self
                            .validator_cache
                            .get_or_insert(validator_key, &json_schema)
                            .await
                            .map_err(|e| NodeError {
                                source: e.into(),
                                node_id: node.id.clone(),
                                trace: error_trace(&node_traces),
                            })?;

                        let context_json = context.to_value();
                        validator.validate(&context_json).map_err(|e| {
                            NodeError {
                                source: anyhow!(
                                    serde_json::to_value(Into::<
                                        Box<EvaluationError>,
                                    >::into(
                                        e
                                    ))
                                    .unwrap_or_default()
                                ),
                                node_id: node.id.clone(),
                                trace: error_trace(&node_traces),
                            }
                        })?;
                    }

                    walker.set_node_data(nid, context.clone());
                },
                // 处理输出节点
                DecisionNodeKind::OutputNode { content } => {
                    let incoming_data =
                        walker.incoming_node_data(&self.graph, nid, false);

                    trace!({
                        input: incoming_data.clone(),
                        output: Variable::Null,
                        trace_data: None,
                    });

                    // 验证输出数据
                    if let Some(json_schema) = content
                        .schema
                        .as_ref()
                        .map(|s| serde_json::from_str::<Value>(&s).ok())
                        .flatten()
                    {
                        let validator_key =
                            create_validator_cache_key(&json_schema);
                        let validator = self
                            .validator_cache
                            .get_or_insert(validator_key, &json_schema)
                            .await
                            .map_err(|e| NodeError {
                                source: e.into(),
                                node_id: node.id.clone(),
                                trace: error_trace(&node_traces),
                            })?;

                        let incoming_data_json = incoming_data.to_value();
                        validator.validate(&incoming_data_json).map_err(
                            |e| NodeError {
                                source: anyhow!(
                                    serde_json::to_value(Into::<
                                        Box<EvaluationError>,
                                    >::into(
                                        e
                                    ))
                                    .unwrap_or_default()
                                ),
                                node_id: node.id.clone(),
                                trace: error_trace(&node_traces),
                            },
                        )?;
                    }

                    return Ok(DecisionGraphResponse {
                        result: incoming_data,
                        performance: format!("{:.1?}", root_start.elapsed()),
                        trace: node_traces,
                    });
                },
                // 处理开关节点
                DecisionNodeKind::SwitchNode { .. } => {
                    let input_data =
                        walker.incoming_node_data(&self.graph, nid, false);
                    walker.set_node_data(nid, input_data);
                },
                // 处理函数节点
                DecisionNodeKind::FunctionNode { content } => {
                    let function = self
                        .get_or_insert_function()
                        .await
                        .map_err(|e| NodeError {
                            source: e.into(),
                            node_id: node.id.clone(),
                            trace: error_trace(&node_traces),
                        })?;

                    let node_request = NodeRequest {
                        node: node.clone(),
                        iteration: self.iteration,
                        input: walker.incoming_node_data(
                            &self.graph,
                            nid,
                            true,
                        ),
                    };

                    // 根据函数版本处理
                    let res = match content {
                        FunctionNodeContent::Version2(_) => {
                            FunctionHandler::new(
                                function,
                                self.trace,
                                self.iteration,
                                self.max_depth,
                            )
                            .handle(node_request.clone())
                            .await
                            .map_err(|e| {
                                if let Some(detailed_err) =
                                    e.downcast_ref::<PartialTraceError>()
                                {
                                    trace!({
                                        input: node_request.input.clone(),
                                        output: Variable::Null,
                                        trace_data: detailed_err.trace.clone(),
                                    });
                                }

                                NodeError {
                                    source: e.into(),
                                    node_id: node.id.clone(),
                                    trace: error_trace(&node_traces),
                                }
                            })?
                        },
                        FunctionNodeContent::Version1(_) => {
                            let runtime =
                                create_runtime().map_err(|e| NodeError {
                                    source: e.into(),
                                    node_id: node.id.clone(),
                                    trace: error_trace(&node_traces),
                                })?;

                            function_v1::FunctionHandler::new(
                                self.trace, runtime,
                            )
                            .handle(node_request.clone())
                            .await
                            .map_err(|e| {
                                NodeError {
                                    source: e.into(),
                                    node_id: node.id.clone(),
                                    trace: error_trace(&node_traces),
                                }
                            })?
                        },
                    };

                    node_request.input.dot_remove("$nodes");
                    res.output.dot_remove("$nodes");

                    trace!({
                        input: node_request.input,
                        output: res.output.clone(),
                        trace_data: res.trace_data,
                    });
                    walker.set_node_data(nid, res.output);
                },
                // 处理决策节点
                DecisionNodeKind::DecisionNode { .. } => {
                    let node_request = NodeRequest {
                        node: node.clone(),
                        iteration: self.iteration,
                        input: walker.incoming_node_data(
                            &self.graph,
                            nid,
                            true,
                        ),
                    };

                    let res = DecisionHandler::new(
                        self.trace,
                        self.max_depth,
                        self.loader.clone(),
                        self.adapter.clone(),
                        self.runtime.clone(),
                        self.validator_cache.clone(),
                    )
                    .handle(node_request.clone())
                    .await
                    .map_err(|e| NodeError {
                        source: e.into(),
                        node_id: node.id.to_string(),
                        trace: error_trace(&node_traces),
                    })?;

                    node_request.input.dot_remove("$nodes");
                    res.output.dot_remove("$nodes");

                    trace!({
                        input: node_request.input,
                        output: res.output.clone(),
                        trace_data: res.trace_data,
                    });
                    walker.set_node_data(nid, res.output);
                },
                // 处理决策表节点
                DecisionNodeKind::DecisionTableNode { .. } => {
                    let node_request = NodeRequest {
                        node: node.clone(),
                        iteration: self.iteration,
                        input: walker.incoming_node_data(
                            &self.graph,
                            nid,
                            true,
                        ),
                    };

                    let res = DecisionTableHandler::new(self.trace)
                        .handle(node_request.clone())
                        .await
                        .map_err(|e| NodeError {
                            node_id: node.id.clone(),
                            source: e.into(),
                            trace: error_trace(&node_traces),
                        })?;

                    node_request.input.dot_remove("$nodes");
                    res.output.dot_remove("$nodes");
                    res.output.dot_remove("$");

                    trace!({
                        input: node_request.input,
                        output: res.output.clone(),
                        trace_data: res.trace_data,
                    });
                    walker.set_node_data(nid, res.output);
                },
                // 处理表达式节点
                DecisionNodeKind::ExpressionNode { .. } => {
                    let node_request = NodeRequest {
                        node: node.clone(),
                        iteration: self.iteration,
                        input: walker.incoming_node_data(
                            &self.graph,
                            nid,
                            true,
                        ),
                    };

                    let res = ExpressionHandler::new(self.trace)
                        .handle(node_request.clone())
                        .await
                        .map_err(|e| {
                            if let Some(detailed_err) =
                                e.downcast_ref::<PartialTraceError>()
                            {
                                trace!({
                                    input: node_request.input.clone(),
                                    output: Variable::Null,
                                    trace_data: detailed_err.trace.clone(),
                                });
                            }

                            NodeError {
                                node_id: node.id.clone(),
                                source: e.into(),
                                trace: error_trace(&node_traces),
                            }
                        })?;

                    node_request.input.dot_remove("$nodes");
                    res.output.dot_remove("$nodes");

                    trace!({
                        input: node_request.input,
                        output: res.output.clone(),
                        trace_data: res.trace_data,
                    });
                    walker.set_node_data(nid, res.output);
                },
                // 处理自定义节点
                DecisionNodeKind::CustomNode { .. } => {
                    let node_request = NodeRequest {
                        node: node.clone(),
                        iteration: self.iteration,
                        input: walker.incoming_node_data(
                            &self.graph,
                            nid,
                            true,
                        ),
                    };

                    let res = self
                        .adapter
                        .handle(
                            CustomNodeRequest::try_from(node_request.clone())
                                .unwrap(),
                        )
                        .await
                        .map_err(|e| NodeError {
                            node_id: node.id.clone(),
                            source: e.into(),
                            trace: error_trace(&node_traces),
                        })?;

                    node_request.input.dot_remove("$nodes");
                    res.output.dot_remove("$nodes");

                    trace!({
                        input: node_request.input,
                        output: res.output.clone(),
                        trace_data: res.trace_data,
                    });
                    walker.set_node_data(nid, res.output);
                },
            }
        }

        // 返回最终结果
        Ok(DecisionGraphResponse {
            result: walker.ending_variables(&self.graph),
            performance: format!("{:.1?}", root_start.elapsed()),
            trace: node_traces,
        })
    }
}

/// 决策图验证错误类型
#[derive(Debug, Error)]
pub enum DecisionGraphValidationError {
    /// 输入节点数量无效
    #[error("无效的输入节点数量: {0}")]
    InvalidInputCount(u32),

    /// 输出节点数量无效
    #[error("无效的输出节点数量: {0}")]
    InvalidOutputCount(u32),

    /// 检测到循环依赖
    #[error("检测到循环依赖")]
    CyclicGraph,

    /// 节点缺失
    #[error("节点缺失: {0}")]
    MissingNode(String),
}

/// 实现序列化特性
impl Serialize for DecisionGraphValidationError {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        match &self {
            DecisionGraphValidationError::InvalidInputCount(count) => {
                map.serialize_entry("type", "invalidInputCount")?;
                map.serialize_entry("nodeCount", count)?;
            },
            DecisionGraphValidationError::InvalidOutputCount(count) => {
                map.serialize_entry("type", "invalidOutputCount")?;
                map.serialize_entry("nodeCount", count)?;
            },
            DecisionGraphValidationError::MissingNode(node_id) => {
                map.serialize_entry("type", "missingNode")?;
                map.serialize_entry("nodeId", node_id)?;
            },
            DecisionGraphValidationError::CyclicGraph => {
                map.serialize_entry("type", "cyclicGraph")?;
            },
        }

        map.end()
    }
}

/// 决策图响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionGraphResponse {
    /// 性能信息
    pub performance: String,
    /// 评估结果
    pub result: Variable,
    /// 可选的跟踪信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<HashMap<String, DecisionGraphTrace>>,
}

/// 决策图跟踪信息结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionGraphTrace {
    /// 输入数据
    pub input: Variable,
    /// 输出数据
    pub output: Variable,
    /// 节点名称
    pub name: String,
    /// 节点ID
    pub id: String,
    /// 性能信息
    pub performance: Option<String>,
    /// 跟踪数据
    pub trace_data: Option<Value>,
    /// 执行顺序
    pub order: u32,
}

/// 将跟踪信息转换为JSON值
pub(crate) fn error_trace(
    trace: &Option<HashMap<String, DecisionGraphTrace>>
) -> Option<Value> {
    trace.as_ref().map(|s| serde_json::to_value(s).ok()).flatten()
}

/// 创建验证器缓存键
fn create_validator_cache_key(content: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}
