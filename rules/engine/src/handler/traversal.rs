// 导入必要的依赖
use ahash::HashMap;
use fixedbitset::FixedBitSet;
use petgraph::data::DataMap;
use petgraph::matrix_graph::Zero;
use petgraph::prelude::{EdgeIndex, NodeIndex, StableDiGraph};
use petgraph::visit::{EdgeRef, IntoNodeIdentifiers, VisitMap, Visitable};
use petgraph::{Incoming, Outgoing};
use serde_json::json;
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use crate::config::ZEN_CONFIG;
use crate::model::{
    DecisionEdge, DecisionNode, DecisionNodeKind, SwitchStatement, SwitchStatementHitPolicy,
};
use crate::DecisionGraphTrace;
use moduforge_rules_expression::variable::Variable;
use moduforge_rules_expression::Isolate;

/// # Petgraph 库说明
/// 
/// Petgraph 是一个功能强大的 Rust 图数据结构库，在本代码中主要用于实现决策图的遍历和管理。
/// 
/// ## 主要功能
/// 1. 图数据结构
///    - 支持有向图和无向图
///    - 提供稳定的图结构（StableDiGraph）
///    - 支持节点和边的权重
/// 
/// ## 核心组件
/// 1. 图结构
///    - `StableDiGraph`: 稳定的有向图实现，保证节点和边的索引在删除操作后保持稳定
///    - `NodeIndex`: 节点索引类型，用于唯一标识图中的节点
///    - `EdgeIndex`: 边索引类型，用于唯一标识图中的边
/// 
/// 2. 遍历功能
///    - `Incoming`: 用于获取节点的入边
///    - `Outgoing`: 用于获取节点的出边
///    - `VisitMap`: 用于跟踪已访问的节点
///    - `IntoNodeIdentifiers`: 用于遍历图中的所有节点
/// 
/// ## 在本代码中的应用
/// 1. 决策图表示
///    - 使用 `StableDiGraph` 存储决策节点和边
///    - 节点存储 `DecisionNode` 信息
///    - 边存储 `DecisionEdge` 信息
/// 
/// 2. 图遍历
///    - 使用 `Incoming` 和 `Outgoing` 遍历节点的依赖关系
///    - 使用 `VisitMap` 跟踪已访问的节点
///    - 使用 `IntoNodeIdentifiers` 获取所有节点
/// 
/// 3. 图操作
///    - 添加和删除节点
///    - 添加和删除边
///    - 查询节点和边的属性
/// 
/// ## 优势
/// 1. 性能
///    - 高效的图操作
///    - 稳定的索引保证
///    - 优化的内存使用
/// 
/// 2. 功能
///    - 丰富的图算法支持
///    - 灵活的数据结构
///    - 良好的类型安全
/// 
/// 3. 可靠性
///    - 经过充分测试
///    - 活跃的维护
///    - 良好的文档支持



/// 定义决策图的类型别名，使用稳定的有向图结构
/// 节点类型为 Arc<DecisionNode>，边类型为 Arc<DecisionEdge>
pub(crate) type StableDiDecisionGraph = StableDiGraph<Arc<DecisionNode>, Arc<DecisionEdge>>;

/// 图遍历器，用于处理决策图的遍历和状态管理
pub(crate) struct GraphWalker {
    /// 记录已访问的节点
    ordered: FixedBitSet,
    /// 待访问的节点队列
    to_visit: Vec<NodeIndex>,
    /// 存储节点数据的映射
    node_data: HashMap<NodeIndex, Variable>,
    /// 当前迭代次数
    iter: usize,
    /// 已访问的开关节点列表
    visited_switch_nodes: Vec<NodeIndex>,
    /// 是否在上下文中包含节点信息
    nodes_in_context: bool,
}

/// 最大迭代次数限制，防止无限循环
const ITER_MAX: usize = 1_000;

impl GraphWalker {
    /// 创建新的图遍历器实例
    /// 初始化遍历器并添加初始节点
    pub fn new(graph: &StableDiDecisionGraph) -> Self {
        let mut topo = Self::empty(graph);
        topo.extend_with_initials(graph);
        topo
    }

    /// 扩展初始节点（没有入边的节点）
    /// 将输入节点添加到待访问队列中
    fn extend_with_initials(&mut self, g: &StableDiDecisionGraph) {
        self.to_visit
            .extend(g.node_identifiers().filter(move |&nid| {
                g.node_weight(nid)
                    .is_some_and(|n| matches!(n.kind, DecisionNodeKind::InputNode { content: _ }))
            }));
    }

    /// 创建空的图遍历器
    /// 初始化所有字段为默认值
    fn empty(graph: &StableDiDecisionGraph) -> Self {
        Self {
            ordered: graph.visit_map(),
            to_visit: Vec::new(),
            node_data: Default::default(),
            visited_switch_nodes: Default::default(),
            iter: 0,
            nodes_in_context: ZEN_CONFIG.nodes_in_context.load(Ordering::Relaxed),
        }
    }

    /// 重置图遍历器状态
    /// 清空已访问节点和待访问队列，重新添加初始节点
    pub fn reset(&mut self, g: &StableDiDecisionGraph) {
        self.ordered.clear();
        self.to_visit.clear();
        self.extend_with_initials(g);
        self.iter += 1;
    }

    /// 获取指定节点的数据
    /// 返回节点的变量数据，如果不存在则返回None
    pub fn get_node_data(&self, node_id: NodeIndex) -> Option<Variable> {
        self.node_data.get(&node_id).cloned()
    }

    /// 获取所有结束节点的变量
    /// 合并所有没有出边的已访问节点的数据
    pub fn ending_variables(&self, g: &StableDiDecisionGraph) -> Variable {
        g.node_indices()
            .filter(|nid| {
                self.ordered.is_visited(nid)
                    && g.neighbors_directed(*nid, Outgoing).count().is_zero()
            })
            .fold(Variable::empty_object(), |mut acc, curr| {
                match self.node_data.get(&curr) {
                    None => acc,
                    Some(data) => acc.merge(data),
                }
            })
    }

    /// 获取所有节点数据
    /// 将节点数据转换为变量对象
    pub fn get_all_node_data(&self, g: &StableDiDecisionGraph) -> Variable {
        let node_values = self
            .node_data
            .iter()
            .filter_map(|(idx, value)| {
                let weight = g.node_weight(*idx)?;
                Some((Rc::from(weight.name.as_str()), value.clone()))
            })
            .collect();

        Variable::from_object(node_values)
    }

    /// 设置节点数据
    /// 将变量数据存储到指定节点
    pub fn set_node_data(&mut self, node_id: NodeIndex, value: Variable) {
        self.node_data.insert(node_id, value);
    }

    /// 获取入边节点的数据
    /// 合并所有入边节点的数据，可选择是否包含节点上下文
    pub fn incoming_node_data(
        &self,
        g: &StableDiDecisionGraph,
        node_id: NodeIndex,
        with_nodes: bool,
    ) -> Variable {
        let value = self
            .merge_node_data(g.neighbors_directed(node_id, Incoming))
            .depth_clone(1);
        if self.nodes_in_context {
            if let Some(object_ref) = with_nodes.then_some(value.as_object()).flatten() {
                let mut object = object_ref.borrow_mut();
                object.insert(Rc::from("$nodes"), self.get_all_node_data(g));
            }
        }
        value
    }

    /// 合并多个节点的数据
    /// 将多个节点的数据合并为一个变量对象
    pub fn merge_node_data<I>(&self, iter: I) -> Variable
    where
        I: Iterator<Item = NodeIndex>,
    {
        let default_map = Variable::empty_object();
        iter.fold(Variable::empty_object(), |mut prev, curr| {
            let data = self.node_data.get(&curr).unwrap_or(&default_map);
            prev.merge_clone(data)
        })
    }

    /// 获取下一个要处理的节点
    /// 
    /// # 功能说明
    /// 实现图遍历的核心逻辑，负责：
    /// 1. 按拓扑顺序遍历决策图
    /// 2. 处理开关节点的条件评估
    /// 3. 移除无效的边和节点
    /// 4. 生成执行跟踪信息
    /// 
    /// # 参数说明
    /// * `g` - 要遍历的决策图
    /// * `on_trace` - 可选的跟踪回调函数，用于记录节点执行信息
    /// 
    /// # 返回值
    /// * `Option<NodeIndex>` - 返回下一个要处理的节点索引，如果没有更多节点则返回None
    /// 
    /// # 处理流程
    /// 1. 检查迭代次数是否超过限制
    /// 2. 从待访问队列中取出节点
    /// 3. 检查节点依赖是否已解析
    /// 4. 处理开关节点的条件评估
    /// 5. 移除无效边和死分支
    /// 6. 添加后继节点到待访问队列
    pub fn next<F: FnMut(DecisionGraphTrace)>(
        &mut self,
        g: &mut StableDiDecisionGraph,
        mut on_trace: Option<F>,
    ) -> Option<NodeIndex> {
        // 记录开始时间，用于性能跟踪
        let start = Instant::now();
        
        // 检查是否超过最大迭代次数限制
        if self.iter >= ITER_MAX {
            return None;
        }

        // 循环处理待访问队列中的节点
        while let Some(nid) = self.to_visit.pop() {
            // 获取当前节点的决策节点数据
            let decision_node = g.node_weight(nid)?.clone();
            
            // 跳过已访问的节点
            if self.ordered.is_visited(&nid) {
                continue;
            }

            // 检查节点的所有依赖是否已解析
            // 如果有未解析的依赖，将当前节点和未解析的依赖重新加入队列
            if !self.all_dependencies_resolved(g, nid) {
                self.to_visit.push(nid);
                self.to_visit
                    .extend(self.get_unresolved_dependencies(g, nid));
                continue;
            }

            // 标记当前节点为已访问
            self.ordered.visit(nid);

            // 处理开关节点
            if let DecisionNodeKind::SwitchNode { content } = &decision_node.kind {
                // 确保每个开关节点只处理一次
                if !self.visited_switch_nodes.contains(&nid) {
                    // 获取输入数据并准备执行环境
                    let input_data = self.incoming_node_data(g, nid, true);
                    let env = input_data.depth_clone(1);
                    env.dot_insert("$", input_data.depth_clone(1));
                    let mut isolate = Isolate::with_environment(env);

                    // 根据命中策略处理开关语句
                    let mut statement_iter = content.statements.iter();
                    let valid_statements: Vec<&SwitchStatement> = match content.hit_policy {
                        // First策略：找到第一个满足条件的语句
                        SwitchStatementHitPolicy::First => statement_iter
                            .find(|&s| switch_statement_evaluate(&mut isolate, &s))
                            .into_iter()
                            .collect(),
                        // Collect策略：收集所有满足条件的语句
                        SwitchStatementHitPolicy::Collect => statement_iter
                            .filter(|&s| switch_statement_evaluate(&mut isolate, &s))
                            .collect(),
                    };

                    // 生成跟踪数据，记录有效的语句ID
                    let valid_statements_trace = Variable::from_array(
                        valid_statements
                            .iter()
                            .map(|&statement| {
                                let v = Variable::empty_object();
                                v.dot_insert(
                                    "id",
                                    Variable::String(Rc::from(statement.id.as_str())),
                                );
                                v
                            })
                            .collect(),
                    );

                    // 移除节点上下文数据
                    input_data.dot_remove("$nodes");

                    // 执行跟踪回调，记录节点执行信息
                    if let Some(on_trace) = &mut on_trace {
                        on_trace(DecisionGraphTrace {
                            id: decision_node.id.clone(),
                            name: decision_node.name.clone(),
                            input: input_data.shallow_clone(),
                            output: input_data.shallow_clone(),
                            order: 0,
                            performance: Some(format!("{:.1?}", start.elapsed())),
                            trace_data: Some(
                                json!({ "statements": valid_statements_trace }).into(),
                            ),
                        });
                    }

                    // 移除无效边
                    // 找出所有不在有效语句列表中的边
                    let edges_to_remove: Vec<EdgeIndex> = g
                        .edges_directed(nid, Outgoing)
                        .filter(|edge| {
                            edge.weight().source_handle.as_ref().map_or(true, |handle| {
                                !valid_statements.iter().any(|s| s.id == *handle)
                            })
                        })
                        .map(|edge| edge.id())
                        .collect();
                    
                    // 记录移除的边数量
                    let edges_remove_count = edges_to_remove.len();
                    
                    // 递归移除无效边及其相关的死分支
                    for edge in edges_to_remove {
                        remove_edge_recursive(g, edge);
                    }

                    // 标记当前开关节点为已访问
                    self.visited_switch_nodes.push(nid);
                    
                    // 如果移除了边，重置图遍历器并继续
                    if edges_remove_count > 0 {
                        self.reset(g);
                        continue;
                    }
                }
            }

            // 将当前节点的所有后继节点添加到待访问队列
            let successors = g.neighbors_directed(nid, Outgoing);
            self.to_visit.extend(successors);

            // 返回当前处理的节点
            return Some(nid);
        }

        // 如果没有更多节点要处理，返回None
        None
    }

    /// 检查节点的所有依赖是否已解析
    /// 确保所有入边节点都已被访问
    fn all_dependencies_resolved(&self, g: &StableDiDecisionGraph, nid: NodeIndex) -> bool {
        g.neighbors_directed(nid, Incoming)
            .all(|dep| self.ordered.is_visited(&dep))
    }

    /// 获取未解析的依赖节点
    /// 返回所有未被访问的入边节点
    fn get_unresolved_dependencies(
        &self,
        g: &StableDiDecisionGraph,
        nid: NodeIndex,
    ) -> Vec<NodeIndex> {
        g.neighbors_directed(nid, Incoming)
            .filter(|dep| !self.ordered.is_visited(dep))
            .collect()
    }
}

/// 评估开关语句的条件
/// 如果条件为空则返回true，否则在隔离环境中执行条件表达式
fn switch_statement_evaluate<'a>(
    isolate: &mut Isolate<'a>,
    switch_statement: &'a SwitchStatement,
) -> bool {
    if switch_statement.condition.is_empty() {
        return true;
    }

    // 直接使用 run_standard，表达式系统会自动使用 thread_local State
    isolate
        .run_standard(switch_statement.condition.as_str())
        .map_or(false, |v| v.as_bool().unwrap_or(false))
}

/// 递归移除边及其相关的死分支
/// 处理目标节点和源节点的死分支，确保图的完整性
fn remove_edge_recursive(g: &mut StableDiDecisionGraph, edge_id: EdgeIndex) {
    let Some((source_nid, target_nid)) = g.edge_endpoints(edge_id) else {
        return;
    };

    g.remove_edge(edge_id);

    // 处理目标节点的死分支
    let target_incoming_count = g.edges_directed(target_nid, Incoming).count();
    if target_incoming_count.is_zero() {
        let edge_ids: Vec<EdgeIndex> = g
            .edges_directed(target_nid, Outgoing)
            .map(|edge| edge.id())
            .collect();

        edge_ids.iter().for_each(|edge_id| {
            remove_edge_recursive(g, edge_id.clone());
        });

        if g.edges(target_nid).count().is_zero() {
            g.remove_node(target_nid);
        }
    }

    // 处理源节点的死分支
    let source_outgoing_count = g.edges_directed(source_nid, Outgoing).count();
    if source_outgoing_count.is_zero() {
        let edge_ids: Vec<EdgeIndex> = g
            .edges_directed(source_nid, Incoming)
            .map(|edge| edge.id())
            .collect();

        edge_ids.iter().for_each(|edge_id| {
            remove_edge_recursive(g, edge_id.clone());
        });

        if g.edges(source_nid).count().is_zero() {
            g.remove_node(source_nid);
        }
    }
}


