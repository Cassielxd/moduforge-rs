use std::collections::{HashMap, HashSet};
use petgraph::algo::is_cyclic_directed;
use petgraph::graph::DiGraph;
use anyhow::Result;

/// 依赖管理器
#[derive(Debug)]
pub struct DependencyManager {
    dependency_graph: DiGraph<String, ()>,
    node_indices: HashMap<String, petgraph::graph::NodeIndex>,
}

impl DependencyManager {
    pub fn new() -> Self {
        Self { dependency_graph: DiGraph::new(), node_indices: HashMap::new() }
    }
    /// 添加插件节点
    pub fn add_plugin(
        &mut self,
        plugin_name: &str,
    ) {
        if !self.node_indices.contains_key(plugin_name) {
            let idx = self.dependency_graph.add_node(plugin_name.to_string());
            self.node_indices.insert(plugin_name.to_string(), idx);
        }
    }
    /// 添加依赖关系
    pub fn add_dependency(
        &mut self,
        dependent: &str,
        dependency: &str,
    ) -> Result<()> {
        // 确保节点存在
        self.add_plugin(dependent);
        self.add_plugin(dependency);

        // 添加边
        let dependent_idx = self.node_indices[dependent];
        let dependency_idx = self.node_indices[dependency];
        self.dependency_graph.add_edge(dependent_idx, dependency_idx, ());

        Ok(())
    }
    /// 检查缺失的依赖 - 直接从图中提取
    pub fn check_missing_dependencies(&self) -> MissingDependencyReport {
        let available_plugins: HashSet<String> =
            self.node_indices.keys().cloned().collect();
        let mut missing_deps = HashMap::new();
        let mut total_missing = 0;

        // 遍历图中的所有边，找出缺失的依赖
        for edge in self.dependency_graph.edge_indices() {
            let (source_idx, target_idx) =
                self.dependency_graph.edge_endpoints(edge).unwrap();
            let dependent = self.dependency_graph[source_idx].clone();
            let dependency = self.dependency_graph[target_idx].clone();

            // 如果依赖节点不存在，说明是缺失的依赖
            if !available_plugins.contains(&dependency) {
                missing_deps
                    .entry(dependent)
                    .or_insert_with(Vec::new)
                    .push(dependency.clone());
                total_missing += 1;
            }
        }

        MissingDependencyReport {
            has_missing_dependencies: !missing_deps.is_empty(),
            total_missing_count: total_missing,
            missing_dependencies: missing_deps,
            available_plugins,
        }
    }
    /// 检查循环依赖
    pub fn has_circular_dependencies(&self) -> bool {
        is_cyclic_directed(&self.dependency_graph)
    }
    /// 获取循环依赖
    pub fn get_circular_dependencies(&self) -> Vec<Vec<String>> {
        if !self.has_circular_dependencies() {
            return vec![];
        }

        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        // 对每个节点进行深度优先搜索
        for node in self.dependency_graph.node_indices() {
            if !visited.contains(&node) {
                self.dfs_cycles(
                    node,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        // 去重和排序
        self.deduplicate_cycles(cycles)
    }
    /// 深度优先搜索查找循环
    fn dfs_cycles(
        &self,
        node: petgraph::graph::NodeIndex,
        visited: &mut HashSet<petgraph::graph::NodeIndex>,
        rec_stack: &mut HashSet<petgraph::graph::NodeIndex>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        let node_name = self.dependency_graph[node].clone();
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node_name.clone());

        // 遍历所有邻居节点
        for neighbor in self.dependency_graph.neighbors(node) {
            if !visited.contains(&neighbor) {
                // 继续深度优先搜索
                self.dfs_cycles(neighbor, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(&neighbor) {
                // 找到循环！neighbor 在递归栈中，说明形成了循环
                let neighbor_name = self.dependency_graph[neighbor].clone();

                // 找到循环的起始位置
                if let Some(start_idx) =
                    path.iter().position(|x| x == &neighbor_name)
                {
                    // 提取循环路径
                    let cycle = path[start_idx..].to_vec();

                    // 确保循环是完整的（首尾相连）
                    if cycle.len() > 1 {
                        cycles.push(cycle);
                    }
                }
            }
        }

        // 回溯：从递归栈和路径中移除当前节点
        rec_stack.remove(&node);
        path.pop();
    }
    /// 去重和排序循环依赖
    fn deduplicate_cycles(
        &self,
        mut cycles: Vec<Vec<String>>,
    ) -> Vec<Vec<String>> {
        if cycles.is_empty() {
            return cycles;
        }

        // 标准化循环：将每个循环旋转到最小字典序
        for cycle in &mut cycles {
            self.normalize_cycle(cycle);
        }

        // 去重
        cycles.sort();
        cycles.dedup();

        cycles
    }
    /// 标准化循环：旋转到最小字典序
    fn normalize_cycle(
        &self,
        cycle: &mut Vec<String>,
    ) {
        if cycle.len() <= 1 {
            return;
        }

        // 找到最小字典序的起始位置
        let min_pos = cycle
            .iter()
            .enumerate()
            .min_by_key(|(_, item)| item.clone())
            .map(|(pos, _)| pos)
            .unwrap_or(0);

        // 旋转到最小字典序位置
        if min_pos > 0 {
            cycle.rotate_left(min_pos);
        }
    }

    /// 获取拓扑排序
    pub fn get_topological_order(&self) -> Result<Vec<String>> {
        if self.has_circular_dependencies() {
            return Err(anyhow::anyhow!("存在循环依赖，无法进行拓扑排序"));
        }

        let order = petgraph::algo::toposort(&self.dependency_graph, None)
            .map_err(|_| anyhow::anyhow!("拓扑排序失败"))?;

        let mut result = Vec::new();
        for node_idx in order {
            result.push(self.dependency_graph[node_idx].clone());
        }

        Ok(result)
    }

    /// 获取插件的直接依赖
    pub fn get_direct_dependencies(
        &self,
        plugin_name: &str,
    ) -> Vec<String> {
        if let Some(&node_idx) = self.node_indices.get(plugin_name) {
            self.dependency_graph
                .neighbors(node_idx)
                .map(|idx| self.dependency_graph[idx].clone())
                .collect()
        } else {
            vec![]
        }
    }
    /// 获取插件的所有依赖（包括间接依赖）
    pub fn get_all_dependencies(
        &self,
        plugin_name: &str,
    ) -> HashSet<String> {
        let mut all_deps = HashSet::new();
        let mut to_visit = std::collections::VecDeque::new();

        to_visit.push_back(plugin_name.to_string());

        while let Some(current) = to_visit.pop_front() {
            if all_deps.insert(current.clone()) {
                let deps = self.get_direct_dependencies(&current);
                to_visit.extend(deps);
            }
        }

        all_deps.remove(plugin_name);
        all_deps
    }
    /// 获取循环依赖的详细报告
    pub fn get_circular_dependency_report(&self) -> CircularDependencyReport {
        let cycles = self.get_circular_dependencies();

        CircularDependencyReport {
            has_circular_dependencies: !cycles.is_empty(),
            cycle_count: cycles.len(),
            cycles: cycles.clone(),
            affected_plugins: self.get_affected_plugins(cycles),
        }
    }
    /// 获取受影响的插件
    fn get_affected_plugins(
        &self,
        cycles: Vec<Vec<String>>,
    ) -> HashSet<String> {
        let mut affected = HashSet::new();

        for cycle in cycles {
            for plugin in cycle {
                affected.insert(plugin.clone());
            }
        }

        affected
    }
}

/// 缺失依赖报告
#[derive(Debug, Clone)]
pub struct MissingDependencyReport {
    pub has_missing_dependencies: bool,
    pub total_missing_count: usize,
    pub missing_dependencies: HashMap<String, Vec<String>>,
    pub available_plugins: HashSet<String>,
}

impl MissingDependencyReport {
    /// 生成人类可读的报告
    pub fn to_string(&self) -> String {
        if !self.has_missing_dependencies {
            return "✅ 所有依赖都已满足".to_string();
        }

        let mut report = String::new();
        report.push_str(&format!(
            "❌ 检测到 {} 个缺失的依赖\n",
            self.total_missing_count
        ));
        report.push_str(&format!("可用插件: {:?}\n", self.available_plugins));
        report.push_str("\n缺失依赖详情:\n");

        for (plugin_name, missing_deps) in &self.missing_dependencies {
            report.push_str(&format!(
                "  {} 缺失依赖: {:?}\n",
                plugin_name, missing_deps
            ));
        }

        report
    }

    /// 获取所有缺失的依赖名称
    pub fn get_all_missing_dependency_names(&self) -> HashSet<String> {
        let mut all_missing = HashSet::new();
        for missing_deps in self.missing_dependencies.values() {
            all_missing.extend(missing_deps.iter().cloned());
        }
        all_missing
    }
}

/// 循环依赖报告
#[derive(Debug, Clone)]
pub struct CircularDependencyReport {
    pub has_circular_dependencies: bool,
    pub cycle_count: usize,
    pub cycles: Vec<Vec<String>>,
    pub affected_plugins: HashSet<String>,
}

impl CircularDependencyReport {
    /// 生成人类可读的报告
    pub fn to_string(&self) -> String {
        if !self.has_circular_dependencies {
            return "✅ 未检测到循环依赖".to_string();
        }

        let mut report = String::new();
        report
            .push_str(&format!("❌ 检测到 {} 个循环依赖\n", self.cycle_count));
        report
            .push_str(&format!("受影响的插件: {:?}\n", self.affected_plugins));
        report.push_str("\n循环依赖详情:\n");

        for (i, cycle) in self.cycles.iter().enumerate() {
            report.push_str(&format!("  循环 {}: ", i + 1));

            for (j, plugin) in cycle.iter().enumerate() {
                if j > 0 {
                    report.push_str(" -> ");
                }
                report.push_str(plugin);
            }

            // 显示循环的闭合
            if !cycle.is_empty() {
                report.push_str(" -> ");
                report.push_str(&cycle[0]);
            }

            report.push('\n');
        }

        report
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dependency_manager() -> Result<()> {
        let mut dependency_manager = DependencyManager::new();

        // 创建更复杂的依赖关系
        // plugin_a -> plugin_b -> plugin_c -> plugin_d -> plugin_b
        // plugin_e -> plugin_f -> plugin_e
        dependency_manager.add_dependency("plugin_a", "plugin_b")?;
        dependency_manager.add_dependency("plugin_b", "plugin_c")?;
        dependency_manager.add_dependency("plugin_c", "plugin_d")?;
        dependency_manager.add_dependency("plugin_d", "plugin_b")?; // 形成循环

        dependency_manager.add_dependency("plugin_e", "plugin_f")?;
        dependency_manager.add_dependency("plugin_f", "plugin_e")?; // 另一个循环

        let report = dependency_manager.get_circular_dependency_report();
        println!("复杂循环依赖报告:\n{}", report.to_string());
        Ok(())
    }
}
