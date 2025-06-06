use std::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use std::cmp::Ordering;

use crate::error::PoolResult;

use super::node::Node;
use super::node_type::NodeType;
use super::schema::Schema;
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MatchEdge {
    pub node_type: NodeType,
    pub next: ContentMatch,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ContentMatch {
    pub next: Vec<MatchEdge>,
    pub wrap_cache: Vec<Option<NodeType>>,
    pub valid_end: bool,
}
impl Ord for ContentMatch {
    fn cmp(
        &self,
        other: &Self,
    ) -> Ordering {
        let _ = other;
        Ordering::Equal
    }
}
impl PartialOrd for ContentMatch {
    fn partial_cmp(
        &self,
        other: &Self,
    ) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl ContentMatch {
    pub fn parse(
        str: String,
        nodes: &HashMap<String, NodeType>,
    ) -> ContentMatch {
        let mut stream = TokenStream::new(str, nodes.clone());
        if stream.next().is_none() {
            return ContentMatch::empty();
        }
        let expr = parse_expr(&mut stream);

        let arr = nfa(expr);

        dfa(arr)
    }
    pub fn empty() -> Self {
        ContentMatch {
            next: Vec::new(),
            wrap_cache: Vec::new(),
            valid_end: true,
        }
    }

    pub fn match_type(
        &self,
        node_type: &NodeType,
    ) -> Option<&ContentMatch> {
        self.next
            .iter()
            .find(|edge| &edge.node_type == node_type)
            .map(|edge| &edge.next)
    }

    pub fn match_fragment(
        &self,
        frag: &[Node],
        schema: &Schema,
    ) -> Option<&ContentMatch> {
        let mut current: &ContentMatch = self;

        for content in frag.iter() {
            if let Some(next) =
                current.match_type(schema.nodes.get(&content.r#type).unwrap())
            {
                current = next;
            } else {
                // 如果无法匹配某个节点类型，返回 None 表示匹配失败
                return None;
            }
        }
        Some(current)
    }

    /// 根据内容匹配规则推导需要的节点类型
    ///
    /// # 参数
    /// - `after`: 待匹配的节点列表
    /// - `to_end`: 是否需要匹配到结束状态
    /// - `schema`: 当前使用的文档模式
    ///
    /// # 返回值
    /// 返回需要的节点类型名称列表，如果无法匹配则返回None
    pub fn fill(
        &self,
        after: &Vec<Node>,
        to_end: bool,
        schema: &Schema,
    ) -> Option<Vec<String>> {
        let mut seen: Vec<ContentMatch> = Vec::new();
        seen.push(self.clone());
        fn search(
            seen: &mut Vec<ContentMatch>,
            to_end: bool,
            after: &Vec<Node>,
            match_: &ContentMatch,
            types: &mut Vec<String>,
            schema: &Schema,
        ) -> Option<Vec<String>> {
            // 首先检查是否可以匹配当前片段
            if let Some(finished) = match_.match_fragment(after, schema) {
                if finished.valid_end || !to_end {
                    return Some(types.clone());
                }
            } else if !after.is_empty() {
                // 如果 after 不为空但无法匹配，直接返回 None
                return None;
            }

            // 然后尝试按顺序匹配每个边
            for edge in &match_.next {
                if !seen.contains(&edge.next) {
                    seen.push(edge.next.clone());
                    types.push(edge.node_type.name.clone());
                    if let Some(found) =
                        search(seen, to_end, after, &edge.next, types, schema)
                    {
                        return Some(found);
                    }
                    types.pop();
                }
            }
            None
        }

        search(&mut seen, to_end, after, self, &mut Vec::new(), schema)
    }

    pub fn default_type(&self) -> Option<&NodeType> {
        self.next
            .iter()
            .find(|edge| !edge.node_type.has_required_attrs())
            .map(|edge| &edge.node_type)
    }

    pub fn compatible(
        &self,
        other: &ContentMatch,
    ) -> bool {
        for edge1 in &self.next {
            for edge2 in &other.next {
                if edge1.node_type == edge2.node_type {
                    return true;
                }
            }
        }
        false
    }

    pub fn edge_count(&self) -> usize {
        self.next.len()
    }

    pub fn edge(
        &self,
        n: usize,
        // 根据错误提示，PoolResult 类型别名可能只接受一个泛型参数，这里修改为只传递一个泛型参数
    ) -> PoolResult<&MatchEdge> {
        if n >= self.next.len() {
            Err(anyhow::anyhow!(format!("{} 超出了 {}", n, self.next.len())))
        } else {
            Ok(&self.next[n])
        }
    }
}
impl fmt::Display for ContentMatch {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        let mut seen = Vec::new();
        fn scan(
            m: &ContentMatch,
            seen: &mut Vec<ContentMatch>,
        ) {
            seen.push(m.clone());
            for edge in &m.next {
                if !seen.iter().any(|s| s == &edge.next) {
                    scan(&edge.next, seen);
                }
            }
        }
        scan(self, &mut seen);

        let str = seen
            .iter()
            .enumerate()
            .map(|(i, m)| {
                let mut out =
                    format!("{} ", if m.valid_end { i + 1 } else { i });
                for (j, edge) in m.next.iter().enumerate() {
                    if j > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&format!(
                        "{}->{}",
                        edge.node_type.name,
                        seen.iter().position(|s| s == &edge.next).unwrap() + 1
                    ));
                }
                out
            })
            .collect::<Vec<_>>()
            .join("\n");

        write!(f, "{}", str)
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TokenStream {
    pos: usize,
    tokens: Vec<String>,
    node_types: HashMap<String, NodeType>,
    string: String,
}

impl TokenStream {
    pub fn new(
        string: String,
        node_types: HashMap<String, NodeType>,
    ) -> Self {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        for c in string.chars() {
            if c.is_whitespace() {
                // 如果当前字符是空白字符，且当前令牌不为空，则将当前令牌添加到令牌列表中
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear(); // 清空当前令牌
                }
            } else if !c.is_alphanumeric() {
                // 如果当前字符是非字母数字字符，且当前令牌不为空，则将当前令牌添加到令牌列表中
                if !current_token.is_empty() {
                    tokens.push(current_token.clone());
                    current_token.clear(); // 清空当前令牌
                }
                // 将非字母数字字符作为单独的令牌添加到列表中
                tokens.push(c.to_string());
            } else {
                // 如果当前字符是字母数字字符，则将其添加到当前令牌中
                current_token.push(c);
            }
        }

        // 如果最后一个令牌不为空，则将其添加到令牌列表中
        if !current_token.is_empty() {
            tokens.push(current_token);
        }
        TokenStream { pos: 0, tokens, node_types, string }
    }

    pub fn next(&self) -> Option<&str> {
        self.tokens.get(self.pos).map(|s| s.as_str())
    }

    pub fn eat(
        &mut self,
        tok: &str,
    ) -> bool {
        if self.next() == Some(tok) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    pub fn err(
        &self,
        str: &str,
    ) -> ! {
        panic!("{} (约束必须是 '{}')", str, self.string);
    }
}

#[derive(Debug, Clone)]
enum Expr {
    Choice { exprs: Vec<Expr> },
    Seq { exprs: Vec<Expr> },
    Plus { expr: Box<Expr> },
    Star { expr: Box<Expr> },
    Opt { expr: Box<Expr> },
    Range { min: usize, max: isize, expr: Box<Expr> },
    Name { value: NodeType },
}
fn parse_expr(stream: &mut TokenStream) -> Expr {
    let mut exprs = Vec::new();

    loop {
        exprs.push(parse_expr_seq(stream));
        if !stream.eat("|") {
            break;
        }
    }
    if exprs.len() == 1 { exprs.pop().unwrap() } else { Expr::Choice { exprs } }
}
fn parse_expr_seq(stream: &mut TokenStream) -> Expr {
    let mut exprs = Vec::new();

    while let Some(next) = stream.next() {
        if next == ")" || next == "|" {
            break;
        }
        exprs.push(parse_expr_subscript(stream));
    }
    if exprs.len() == 1 { exprs.pop().unwrap() } else { Expr::Seq { exprs } }
}

fn parse_expr_subscript(stream: &mut TokenStream) -> Expr {
    let mut expr = parse_expr_atom(stream);
    loop {
        if stream.eat("+") {
            expr = Expr::Plus { expr: Box::new(expr) };
        } else if stream.eat("*") {
            expr = Expr::Star { expr: Box::new(expr) };
        } else if stream.eat("?") {
            expr = Expr::Opt { expr: Box::new(expr) };
        } else if stream.eat("{") {
            expr = parse_expr_range(stream, expr);
        } else {
            break;
        }
    }
    expr
}

fn parse_num(stream: &mut TokenStream) -> usize {
    let next = stream.next().unwrap();
    if !next.chars().all(|c| c.is_ascii_digit()) {
        stream.err(&format!("Expected number, got '{}'", next));
    }
    let result = next.parse().unwrap();
    stream.pos += 1;
    result
}
fn parse_expr_range(
    stream: &mut TokenStream,
    expr: Expr,
) -> Expr {
    let min = parse_num(stream);
    let max = if stream.eat(",") {
        if stream.next() != Some("}") { parse_num(stream) as isize } else { -1 }
    } else {
        min as isize
    };
    if !stream.eat("}") {
        stream.err("Unclosed braced range");
    }
    Expr::Range { min, max, expr: Box::new(expr) }
}

fn resolve_name(
    stream: &TokenStream,
    name: &str,
) -> Vec<NodeType> {
    let types = &stream.node_types;
    if let Some(type_) = types.get(name) {
        return vec![type_.clone()];
    }
    let mut result = Vec::new();

    for type_ in types.values() {
        if type_.groups.contains(&name.to_string()) {
            result.push(type_.clone());
        }
    }
    if result.is_empty() {
        stream.err(&format!("没找到类型 '{}'", name));
    }
    result
}

fn parse_expr_atom(stream: &mut TokenStream) -> Expr {
    if stream.eat("(") {
        let expr = parse_expr(stream);
        if !stream.eat(")") {
            stream.err("Missing closing paren");
        }
        expr
    } else if let Some(next) = stream.next() {
        if next.chars().all(|c| c.is_alphanumeric()) {
            let exprs: Vec<Expr> = resolve_name(stream, next)
                .into_iter()
                .map(|type_| Expr::Name { value: type_ })
                .collect();
            stream.pos += 1;
            if exprs.len() == 1 {
                exprs.into_iter().next().unwrap()
            } else {
                Expr::Choice { exprs }
            }
        } else {
            stream.err(&format!("Unexpected token '{}'", next));
        }
    } else {
        stream.err("Unexpected end of input");
    }
}
#[derive(Debug, Clone)]
pub struct Edge {
    term: Option<NodeType>,
    to: Option<usize>,
}
fn dfa(nfa: Vec<Vec<Rc<RefCell<Edge>>>>) -> ContentMatch {
    let mut labeled: HashMap<String, ContentMatch> = HashMap::new();

    fn explore(
        states: Vec<usize>,
        nfa: &Vec<Vec<Rc<RefCell<Edge>>>>,
        labeled: &mut HashMap<String, ContentMatch>,
    ) -> ContentMatch {
        let mut out: Vec<(NodeType, Vec<usize>)> = Vec::new();
        for &node in &states {
            for edge in &nfa[node] {
                if edge.borrow().term.is_none() {
                    continue;
                }
                let term = edge.borrow().term.clone().unwrap();
                let mut set: Option<&mut Vec<usize>> = None;

                for (t, s) in &mut out {
                    if *t == term {
                        set = Some(s);
                        break;
                    }
                }

                if set.is_none() {
                    out.push((term.clone(), Vec::new()));
                    set = Some(&mut out.last_mut().unwrap().1);
                }
                for &node in &null_from(nfa, edge.borrow().to.unwrap_or(0)) {
                    set.as_mut().unwrap().push(node);
                }
            }
        }
        let mut state = ContentMatch {
            next: Vec::new(),
            wrap_cache: vec![],
            valid_end: states.contains(&(nfa.len() - 1)),
        };

        let state_key =
            states.iter().map(|&x| x.to_string()).collect::<Vec<_>>().join(",");
        labeled.insert(state_key.clone(), state.clone());

        for (term, states) in out {
            let states_key = states
                .iter()
                .map(|&x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let next_state = labeled
                .get(&states_key)
                .cloned()
                .unwrap_or_else(|| explore(states, nfa, labeled));
            labeled.insert(states_key, next_state.clone());
            state.next.push(MatchEdge { node_type: term, next: next_state });
        }

        state
    }

    explore(null_from(&nfa, 0), &nfa, &mut labeled)
}

pub fn null_from(
    nfa: &[Vec<Rc<RefCell<Edge>>>],
    node: usize,
) -> Vec<usize> {
    let mut result = Vec::new();
    fn scan(
        nfa: &[Vec<Rc<RefCell<Edge>>>],
        node: usize,
        result: &mut Vec<usize>,
    ) {
        let edges = &nfa[node];
        if edges.len() == 1 && edges[0].borrow().term.is_none() {
            if let Some(to) = edges[0].borrow().to {
                scan(nfa, to, result);
            }
            return;
        }
        if !result.contains(&node) {
            result.push(node);
        }
        for edge in edges {
            if edge.borrow().term.is_none() {
                if let Some(to) = edge.borrow().to {
                    if !result.contains(&to) {
                        scan(nfa, to, result);
                    }
                }
            }
        }
    }

    scan(nfa, node, &mut result);
    result.sort();
    result
}
fn nfa(expr: Expr) -> Vec<Vec<Rc<RefCell<Edge>>>> {
    let mut nfa: Vec<Vec<Rc<RefCell<Edge>>>> = vec![vec![]];
    connect(&mut compile(expr, 0, &mut nfa), node(&mut nfa));
    nfa
}
fn node(nfa: &mut Vec<Vec<Rc<RefCell<Edge>>>>) -> usize {
    nfa.push(vec![]);
    nfa.len() - 1
}

fn edge(
    from: usize,
    to: Option<usize>,
    term: Option<NodeType>,
    nfa: &mut [Vec<Rc<RefCell<Edge>>>],
) -> Rc<RefCell<Edge>> {
    let edge =
        Rc::new(RefCell::new(Edge { term, to: Option::from(to.unwrap_or(0)) }));
    nfa[from].push(edge.clone());
    edge.clone()
}
fn connect(
    edges: &mut [Rc<RefCell<Edge>>],
    to: usize,
) {
    for edge in edges {
        edge.borrow_mut().to = Some(to);
    }
}
fn compile(
    expr: Expr,
    from: usize,
    nfa: &mut Vec<Vec<Rc<RefCell<Edge>>>>,
) -> Vec<Rc<RefCell<Edge>>> {
    match expr {
        Expr::Choice { exprs } => exprs
            .into_iter()
            .flat_map(|expr| compile(expr, from, nfa))
            .collect(),
        Expr::Seq { exprs } => {
            let mut cur = from;
            let mut last_edges = Vec::new();
            let exprs_len = exprs.len();

            for (i, expr) in exprs.into_iter().enumerate() {
                let next = if i == exprs_len - 1 { cur } else { node(nfa) };

                let mut edges = compile(expr, cur, nfa);
                if i < exprs_len - 1 {
                    connect(&mut edges, next);
                    cur = next;
                } else {
                    last_edges = edges;
                }
            }

            if last_edges.is_empty() {
                vec![edge(cur, None, None, nfa)]
            } else {
                last_edges
            }
        },
        Expr::Star { expr } => {
            let loop_node = node(nfa);
            edge(from, Some(loop_node), None, nfa);
            let mut compiled_expr = compile(*expr, loop_node, nfa);
            connect(&mut compiled_expr, loop_node);
            vec![edge(loop_node, None, None, nfa)]
        },
        Expr::Plus { expr } => {
            let loop_node = node(nfa);
            connect(&mut compile(*expr.clone(), from, nfa), loop_node);
            let mut compiled_expr = compile(*expr, loop_node, nfa);
            connect(&mut compiled_expr, loop_node);
            vec![edge(loop_node, None, None, nfa)]
        },
        Expr::Opt { expr } => {
            let mut edges = vec![edge(from, None, None, nfa)];
            edges.extend(compile(*expr, from, nfa));
            edges
        },
        Expr::Range { expr, min, max } => {
            let mut cur = from;
            for _ in 0..min {
                let next = node(nfa);
                connect(&mut compile(*expr.clone(), cur, nfa), next);
                cur = next;
            }
            if max == -1 {
                connect(&mut compile(*expr, cur, nfa), cur);
            } else {
                for _ in min..max as usize {
                    let next = node(nfa);
                    edge(cur, Some(next), None, nfa);
                    connect(&mut compile(*expr.clone(), cur, nfa), next);
                    cur = next;
                }
            }
            vec![edge(cur, None, None, nfa)]
        },
        Expr::Name { value } => {
            vec![edge(from, None, Some(value), nfa)]
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{AttributeSpec, Schema, SchemaSpec};
    use crate::node_type::NodeSpec;
    use std::collections::HashMap;
    use serde_json::Value;

    #[test]
    fn test_tablerow_plus_fill() {
        // 创建一个简单的 schema
        let mut nodes = HashMap::new();
        
        // 定义 table 节点：内容为 "tablerow+"
        nodes.insert("table".to_string(), NodeSpec {
            content: Some("tablerow+".to_string()),
            marks: None,
            group: None,
            desc: Some("表格节点".to_string()),
            attrs: None,
        });
        
        // 定义 tablerow 节点
        nodes.insert("tablerow".to_string(), NodeSpec {
            content: Some("tablecell+".to_string()),
            marks: None,
            group: None,
            desc: Some("表格行节点".to_string()),
            attrs: None,
        });
        
        // 定义 tablecell 节点
        nodes.insert("tablecell".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None,
            group: None,
            desc: Some("表格单元格节点".to_string()),
            attrs: None,
        });
        
        // 定义 text 节点
        nodes.insert("text".to_string(), NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("文本节点".to_string()),
            attrs: None,
        });

        let schema_spec = SchemaSpec {
            nodes,
            marks: HashMap::new(),
            top_node: Some("table".to_string()),
        };
        
        let schema = Schema::compile(schema_spec).unwrap();
        let table_type = schema.nodes.get("table").unwrap();
        
        // 测试：当 table 的内容为空时，fill 应该返回至少一个 tablerow
        if let Some(content_match) = &table_type.content_match {
            println!("Table content match: {}", content_match);
            
            // 测试空内容的情况
            let empty_content: Vec<Node> = vec![];
            let result = content_match.fill(&empty_content, true, &schema);
            
            println!("Fill result for empty content: {:?}", result);
            
            if let Some(needed_types) = result {
                println!("成功！需要的节点类型数量: {}", needed_types.len());
                for (i, type_name) in needed_types.iter().enumerate() {
                    println!("  第{}个需要的节点类型: {}", i + 1, type_name);
                }
            } else {
                println!("填充返回了 None");
            }
        }
    }

    #[test]
    fn test_table_create_and_fill() {
        use crate::node_type::NodeType;
        
        // 创建一个简单的 schema
        let mut nodes = HashMap::new();
        
        // 定义 table 节点：内容为 "tablerow+"
        nodes.insert("table".to_string(), NodeSpec {
            content: Some("tablerow+".to_string()),
            marks: None,
            group: None,
            desc: Some("表格节点".to_string()),
            attrs: None,
        });
        
        // 定义 tablerow 节点
        nodes.insert("tablerow".to_string(), NodeSpec {
            content: Some("tablecell+".to_string()),
            marks: None,
            group: None,
            desc: Some("表格行节点".to_string()),
            attrs: None,
        });
        
        // 定义 tablecell 节点
        nodes.insert("tablecell".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None,
            group: None,
            desc: Some("表格单元格节点".to_string()),
            attrs: None,
        });
        
        // 定义 text 节点
        nodes.insert("text".to_string(), NodeSpec {
            content: None,
            marks: None,
            group: None,
            desc: Some("文本节点".to_string()),
            attrs: None,
        });

        let schema_spec = SchemaSpec {
            nodes,
            marks: HashMap::new(),
            top_node: Some("table".to_string()),
        };
        
        let schema = Schema::compile(schema_spec).unwrap();
        let table_type = schema.nodes.get("table").unwrap();
        
        // 测试 create_and_fill 与空内容
        println!("=== 测试 create_and_fill ===");
        let empty_content: Vec<Node> = vec![];
        let result = table_type.create_and_fill(
            None,           // id
            None,           // attrs  
            empty_content,  // content
            None,           // marks
            &schema,        // schema
        );
        
        let (main_node, child_nodes) = result.into_parts();
        println!("Main node: {:?}", main_node);
        println!("Child nodes count: {}", child_nodes.len());
        
        for (i, child) in child_nodes.iter().enumerate() {
            let (child_node, grandchildren) = child.clone().into_parts();
            println!("  Child {}: type={}, id={}", i + 1, child_node.r#type, child_node.id);
            println!("    Grandchildren count: {}", grandchildren.len());
        }
        
        // 验证是否创建了 tablerow
        if !child_nodes.is_empty() {
            let (first_child, _) = child_nodes[0].clone().into_parts();
            println!("第一个子节点类型: {}", first_child.r#type);
        } else {
            println!("警告：没有创建任何子节点！");
        }
    }

    #[test]
    fn test_edge_cases() {
        use crate::node_type::NodeType;
        use crate::node::Node;
        use crate::attrs::Attrs;
        
        // 创建schema（与上面相同）
        let mut nodes = HashMap::new();
        nodes.insert("table".to_string(), NodeSpec {
            content: Some("tablerow+".to_string()),
            marks: None, group: None, desc: Some("表格节点".to_string()), attrs: None,
        });
        nodes.insert("tablerow".to_string(), NodeSpec {
            content: Some("tablecell+".to_string()),
            marks: None, group: None, desc: Some("表格行节点".to_string()), attrs: None,
        });
        nodes.insert("tablecell".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None, group: None, desc: Some("表格单元格节点".to_string()), attrs: None,
        });
        nodes.insert("text".to_string(), NodeSpec {
            content: None,
            marks: None, group: None, desc: Some("文本节点".to_string()), attrs: None,
        });

        let schema_spec = SchemaSpec {
            nodes, marks: HashMap::new(), top_node: Some("table".to_string()),
        };
        let schema = Schema::compile(schema_spec).unwrap();
        let table_type = schema.nodes.get("table").unwrap();
        
        println!("=== 边界情况测试 ===");
        
        // 情况1: content_match 为 None
        if table_type.content_match.is_none() {
            println!("警告：table_type.content_match 为 None");
            return;
        }
        let content_match = table_type.content_match.as_ref().unwrap();
        
        // 情况2: match_fragment 返回 None
        let empty_content: Vec<Node> = vec![];
        let matched = content_match.match_fragment(&empty_content, &schema);
        println!("match_fragment result: {:?}", matched.is_some());
        
        if let Some(matched_state) = matched {
            println!("matched state valid_end: {}", matched_state.valid_end);
            
            // 情况3: fill 返回 None
            let fill_result = matched_state.fill(&empty_content, true, &schema);
            println!("fill result: {:?}", fill_result.is_some());
            
            if let Some(needed_types) = fill_result {
                println!("需要的类型数量: {}", needed_types.len());
                for type_name in &needed_types {
                    println!("  需要的类型: {}", type_name);
                }
            }
        }
        
        // 情况4: 测试with to_end=false
        if let Some(matched_state) = matched {
            let fill_result_no_end = matched_state.fill(&empty_content, false, &schema);
            println!("fill result (to_end=false): {:?}", fill_result_no_end.is_some());
        }
    }

    #[test]
    fn test_block_choice_problem() {
        use crate::node_type::NodeType;
        
        // 模拟 simple_demo.rs 中的问题场景
        let mut nodes = HashMap::new();
        
        // 定义 block 节点：内容为 "table paragraph list heading" (选择表达式)
        nodes.insert("block".to_string(), NodeSpec {
            content: Some("table paragraph list heading".to_string()),
            marks: None,
            group: None,
            desc: Some("块级节点".to_string()),
            attrs: None,
        });
        
        // 定义其他节点
        nodes.insert("table".to_string(), NodeSpec {
            content: Some("tablerow+".to_string()),
            marks: None, group: None, desc: Some("表格节点".to_string()), attrs: None,
        });
        nodes.insert("paragraph".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None, group: None, desc: Some("段落节点".to_string()), attrs: None,
        });
        nodes.insert("list".to_string(), NodeSpec {
            content: Some("listitem+".to_string()),
            marks: None, group: None, desc: Some("列表节点".to_string()), attrs: None,
        });
        nodes.insert("heading".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None, group: None, desc: Some("标题节点".to_string()), attrs: None,
        });
        nodes.insert("tablerow".to_string(), NodeSpec {
            content: Some("tablecell+".to_string()),
            marks: None, group: None, desc: Some("表格行节点".to_string()), attrs: None,
        });
        nodes.insert("tablecell".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None, group: None, desc: Some("表格单元格节点".to_string()), attrs: None,
        });
        nodes.insert("listitem".to_string(), NodeSpec {
            content: Some("paragraph".to_string()),
            marks: None, group: None, desc: Some("列表项节点".to_string()), attrs: None,
        });
        nodes.insert("text".to_string(), NodeSpec {
            content: None,
            marks: None, group: None, desc: Some("文本节点".to_string()), attrs: None,
        });

        let schema_spec = SchemaSpec {
            nodes,
            marks: HashMap::new(),
            top_node: Some("block".to_string()),
        };
        
        let schema = Schema::compile(schema_spec).unwrap();
        let block_type = schema.nodes.get("block").unwrap();
        
        println!("=== 测试 Block 选择问题 ===");
        
        if let Some(content_match) = &block_type.content_match {
            println!("Block content match: {}", content_match);
            
            // 检查默认类型
            let default_type = content_match.default_type();
            if let Some(def_type) = default_type {
                println!("默认类型: {}", def_type.name);
                println!("默认类型是否有必须属性: {}", def_type.has_required_attrs());
            } else {
                println!("没有默认类型");
            }
            
            // 测试空内容的填充
            let empty_content: Vec<Node> = vec![];
            let result = content_match.fill(&empty_content, true, &schema);
            
            println!("Fill result: {:?}", result.is_some());
            if let Some(needed_types) = result {
                println!("需要的类型数量: {}", needed_types.len());
                for (i, type_name) in needed_types.iter().enumerate() {
                    println!("  第{}个需要的节点类型: {}", i + 1, type_name);
                }
            }
            
            // 测试 create_and_fill
            println!("=== 测试 Block create_and_fill ===");
            let result = block_type.create_and_fill(
                None,
                None,
                vec![],
                None,
                &schema,
            );
            
            let (main_node, child_nodes) = result.into_parts();
            println!("Main node type: {}", main_node.r#type);
            println!("Child nodes count: {}", child_nodes.len());
            
            for (i, child) in child_nodes.iter().enumerate() {
                let (child_node, grandchildren) = child.clone().into_parts();
                println!("  Child {}: type={}", i + 1, child_node.r#type);
                if child_node.r#type == "table" {
                    println!("    Table 的 grandchildren count: {}", grandchildren.len());
                    for (j, grandchild) in grandchildren.iter().enumerate() {
                        let (gc_node, _) = grandchild.clone().into_parts();
                        println!("      Grandchild {}: type={}", j + 1, gc_node.r#type);
                    }
                }
            }
        }
    }

    #[test]
    fn test_sequence_with_existing_nodes() {
        use crate::node_type::NodeType;
        use crate::node::Node;
        use crate::attrs::Attrs;
        
        // 创建 schema（与上面相同）
        let mut nodes = HashMap::new();
        nodes.insert("block".to_string(), NodeSpec {
            content: Some("table paragraph".to_string()), // 简化的序列
            marks: None, group: None, desc: Some("块级节点".to_string()), attrs: None,
        });
        nodes.insert("table".to_string(), NodeSpec {
            content: Some("tablerow+".to_string()),
            marks: None, group: None, desc: Some("表格节点".to_string()), attrs: None,
        });
        nodes.insert("paragraph".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None, group: None, desc: Some("段落节点".to_string()), attrs: None,
        });
        nodes.insert("tablerow".to_string(), NodeSpec {
            content: Some("tablecell+".to_string()),
            marks: None, group: None, desc: Some("表格行节点".to_string()), attrs: None,
        });
        nodes.insert("tablecell".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None, group: None, desc: Some("表格单元格节点".to_string()), attrs: None,
        });
        nodes.insert("text".to_string(), NodeSpec {
            content: None,
            marks: None, group: None, desc: Some("文本节点".to_string()), attrs: None,
        });

        let schema_spec = SchemaSpec {
            nodes, marks: HashMap::new(), top_node: Some("block".to_string()),
        };
        let schema = Schema::compile(schema_spec).unwrap();
        let block_type = schema.nodes.get("block").unwrap();
        
        println!("=== 测试序列中已存在节点的情况 ===");
        
        // 创建一个已存在的 table 节点
        let existing_table = Node::new(
            "existing_table_123",
            "table".to_string(),
            Attrs::default(),
            vec!["existing_row_456".to_string()],
            vec![]
        );
        
        // 创建一个已存在的 paragraph 节点  
        let existing_paragraph = Node::new(
            "existing_para_789",
            "paragraph".to_string(),
            Attrs::default(),
            vec!["existing_text_000".to_string()],
            vec![]
        );
        
        let existing_content = vec![existing_table, existing_paragraph];
        
        println!("传入的现有内容:");
        for (i, node) in existing_content.iter().enumerate() {
            println!("  第{}个现有节点: type={}, id={}, content={:?}", 
                i + 1, node.r#type, node.id, node.content);
        }
        
        // 测试 create_and_fill 对已存在节点的处理
        let result = block_type.create_and_fill(
            None,
            None,
            existing_content,
            None,
            &schema,
        );
        
        let (main_node, child_nodes) = result.into_parts();
        println!("Main node: type={}, content={:?}", main_node.r#type, main_node.content);
        println!("Child nodes count: {}", child_nodes.len());
        
        for (i, child) in child_nodes.iter().enumerate() {
            let (child_node, grandchildren) = child.clone().into_parts();
            println!("  Child {}: type={}, id={}", i + 1, child_node.r#type, child_node.id);
            println!("    Content: {:?}", child_node.content);
            println!("    Grandchildren count: {}", grandchildren.len());
            
            for (j, grandchild) in grandchildren.iter().enumerate() {
                let (gc_node, _) = grandchild.clone().into_parts();
                println!("      Grandchild {}: type={}, id={}", j + 1, gc_node.r#type, gc_node.id);
            }
        }
    }

    #[test]
    fn test_table_creation_step_by_step() {
        use crate::node_type::NodeType;
        
        // 创建一个简单的 schema，只有 table 和 tablerow
        let mut nodes = HashMap::new();
        
        nodes.insert("table".to_string(), NodeSpec {
            content: Some("tablerow+".to_string()),
            marks: None, group: None, desc: Some("表格节点".to_string()), attrs: None,
        });
        nodes.insert("tablerow".to_string(), NodeSpec {
            content: Some("tablecell+".to_string()),
            marks: None, group: None, desc: Some("表格行节点".to_string()), attrs: None,
        });
        nodes.insert("tablecell".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None, group: None, desc: Some("表格单元格节点".to_string()), attrs: None,
        });
        nodes.insert("text".to_string(), NodeSpec {
            content: None,
            marks: None, group: None, desc: Some("文本节点".to_string()), attrs: None,
        });

        let schema_spec = SchemaSpec {
            nodes, marks: HashMap::new(), top_node: Some("table".to_string()),
        };
        let schema = Schema::compile(schema_spec).unwrap();
        let table_type = schema.nodes.get("table").unwrap();
        
        println!("=== 逐步诊断 Table 创建过程 ===");
        
        // 第1步：检查 content_match
        println!("第1步：检查 table 的 content_match");
        if let Some(content_match) = &table_type.content_match {
            println!("  ✅ content_match 存在");
            println!("  content_match: {}", content_match);
        } else {
            println!("  ❌ content_match 不存在");
            return;
        }
        
        // 第2步：测试 match_fragment
        println!("第2步：测试 match_fragment");
        let empty_content: Vec<Node> = vec![];
        let content_match = table_type.content_match.as_ref().unwrap();
        
        let matched = content_match.match_fragment(&empty_content, &schema);
        if let Some(matched_state) = matched {
            println!("  ✅ match_fragment 成功");
            println!("  matched state valid_end: {}", matched_state.valid_end);
        } else {
            println!("  ❌ match_fragment 返回 None");
            return;
        }
        
        // 第3步：测试 fill
        println!("第3步：测试 fill");
        let matched_state = matched.unwrap();
        let fill_result = matched_state.fill(&empty_content, true, &schema);
        
        if let Some(needed_types) = fill_result {
            println!("  ✅ fill 成功，需要的类型数量: {}", needed_types.len());
            for (i, type_name) in needed_types.iter().enumerate() {
                println!("    第{}个需要的类型: {}", i + 1, type_name);
            }
        } else {
            println!("  ❌ fill 返回 None");
            return;
        }
        
        // 第4步：测试完整的 create_and_fill
        println!("第4步：测试完整的 create_and_fill");
        let result = table_type.create_and_fill(
            None,
            None,
            vec![], // 空内容
            None,
            &schema,
        );
        
        let (main_node, child_nodes) = result.into_parts();
        println!("  Main table node:");
        println!("    ID: {}", main_node.id);
        println!("    Content IDs: {:?}", main_node.content);
        println!("  Child nodes count: {}", child_nodes.len());
        
        if child_nodes.is_empty() {
            println!("  ❌ 没有创建子节点！");
        } else {
            for (i, child) in child_nodes.iter().enumerate() {
                let (child_node, grandchildren) = child.clone().into_parts();
                println!("    Child {}: type={}, id={}", i + 1, child_node.r#type, child_node.id);
                println!("      Content IDs: {:?}", child_node.content);
                println!("      Grandchildren count: {}", grandchildren.len());
                
                if child_node.r#type == "tablerow" && grandchildren.is_empty() {
                    println!("      ❌ tablerow 没有创建 tablecell 子节点！");
                }
                
                for (j, grandchild) in grandchildren.iter().enumerate() {
                    let (gc_node, great_grandchildren) = grandchild.clone().into_parts();
                    println!("        Grandchild {}: type={}, id={}", j + 1, gc_node.r#type, gc_node.id);
                    println!("          Content IDs: {:?}", gc_node.content);
                    println!("          Great-grandchildren count: {}", great_grandchildren.len());
                }
            }
        }
        
        // 第5步：单独测试 tablerow 的创建
        println!("第5步：单独测试 tablerow 的创建");
        let tablerow_type = schema.nodes.get("tablerow").unwrap();
        let tablerow_result = tablerow_type.create_and_fill(
            None,
            None,
            vec![], // 空内容
            None,
            &schema,
        );
        
        let (tr_node, tr_children) = tablerow_result.into_parts();
        println!("  Tablerow node:");
        println!("    ID: {}", tr_node.id);
        println!("    Content IDs: {:?}", tr_node.content);
        println!("    Children count: {}", tr_children.len());
        
        if tr_children.is_empty() {
            println!("    ❌ tablerow 没有创建 tablecell 子节点！");
        } else {
            for (i, child) in tr_children.iter().enumerate() {
                let (child_node, _) = child.clone().into_parts();
                println!("      Child {}: type={}, id={}", i + 1, child_node.r#type, child_node.id);
            }
        }
    }

    #[test]
    fn test_sequence_table_problem() {
        use crate::node_type::NodeType;
        
        // 重现 "table paragraph list heading" 序列表达式的问题
        let mut nodes = HashMap::new();
        
        // 定义 block 节点：内容为 "table paragraph list heading" (序列表达式)
        nodes.insert("block".to_string(), NodeSpec {
            content: Some("table paragraph list heading".to_string()),
            marks: None,
            group: None,
            desc: Some("块级节点".to_string()),
            attrs: None,
        });
        
        // 定义各个子节点
        nodes.insert("table".to_string(), NodeSpec {
            content: Some("tablerow+".to_string()),
            marks: None, group: None, desc: Some("表格节点".to_string()), attrs: None,
        });
        nodes.insert("paragraph".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None, group: None, desc: Some("段落节点".to_string()), attrs: None,
        });
        nodes.insert("list".to_string(), NodeSpec {
            content: Some("listitem+".to_string()),
            marks: None, group: None, desc: Some("列表节点".to_string()), attrs: None,
        });
        nodes.insert("heading".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None, group: None, desc: Some("标题节点".to_string()), attrs: None,
        });
        nodes.insert("tablerow".to_string(), NodeSpec {
            content: Some("tablecell+".to_string()),
            marks: None, group: None, desc: Some("表格行节点".to_string()), attrs: None,
        });
        nodes.insert("tablecell".to_string(), NodeSpec {
            content: Some("text*".to_string()),
            marks: None, group: None, desc: Some("表格单元格节点".to_string()), attrs: None,
        });
        nodes.insert("listitem".to_string(), NodeSpec {
            content: Some("paragraph".to_string()),
            marks: None, group: None, desc: Some("列表项节点".to_string()), attrs: None,
        });
        nodes.insert("text".to_string(), NodeSpec {
            content: None,
            marks: None, group: None, desc: Some("文本节点".to_string()), attrs: None,
        });

        let schema_spec = SchemaSpec {
            nodes,
            marks: HashMap::new(),
            top_node: Some("block".to_string()),
        };
        
        let schema = Schema::compile(schema_spec).unwrap();
        let block_type = schema.nodes.get("block").unwrap();
        
        println!("=== 测试序列表达式中的 Table 问题 ===");
        
        // 创建 block 节点
        let result = block_type.create_and_fill(
            None,
            None,
            vec![], // 空内容，让 fill 方法推导所需节点
            None,
            &schema,
        );
        
        let (main_node, child_nodes) = result.into_parts();
        println!("Block 节点:");
        println!("  ID: {}", main_node.id);
        println!("  Content IDs: {:?}", main_node.content);
        println!("  子节点数量: {}", child_nodes.len());
        
        // 检查每个子节点
        for (i, child) in child_nodes.iter().enumerate() {
            let (child_node, grandchildren) = child.clone().into_parts();
            println!("  子节点 {}: type={}, id={}", i + 1, child_node.r#type, child_node.id);
            println!("    Content IDs: {:?}", child_node.content);
            println!("    孙节点数量: {}", grandchildren.len());
            
            // 特别检查 table 节点
            if child_node.r#type == "table" {
                println!("    📋 这是 Table 节点:");
                
                // 检查 table 的直接子节点 IDs
                if child_node.content.is_empty() {
                    println!("      ❌ Table 节点的 content IDs 是空的！");
                } else {
                    println!("      ✅ Table 节点包含 content IDs: {:?}", child_node.content);
                }
                
                // 检查 table 的孙节点（tablerow）
                if grandchildren.is_empty() {
                    println!("      ❌ Table 节点没有创建任何孙节点（tablerow）！");
                } else {
                    println!("      ✅ Table 节点创建了 {} 个孙节点:", grandchildren.len());
                    for (j, grandchild) in grandchildren.iter().enumerate() {
                        let (gc_node, great_grandchildren) = grandchild.clone().into_parts();
                        println!("        孙节点 {}: type={}, id={}", j + 1, gc_node.r#type, gc_node.id);
                        println!("          Content IDs: {:?}", gc_node.content);
                        
                        // 检查 tablerow 的子节点（tablecell）
                        if gc_node.r#type == "tablerow" {
                            if great_grandchildren.is_empty() {
                                println!("          ❌ tablerow 没有创建 tablecell 子节点！");
                                
                                // 深入调试 tablerow 的填充过程
                                println!("          🔍 调试 tablerow 填充过程:");
                                let tablerow_type = schema.nodes.get("tablerow").unwrap();
                                if let Some(tr_content_match) = &tablerow_type.content_match {
                                    println!("            tablerow content_match: {}", tr_content_match);
                                    
                                    let empty_content: Vec<Node> = vec![];
                                    let tr_matched = tr_content_match.match_fragment(&empty_content, &schema);
                                    if let Some(tr_matched_state) = tr_matched {
                                        println!("            tablerow match_fragment 成功");
                                        println!("            tablerow matched state valid_end: {}", tr_matched_state.valid_end);
                                        
                                        let tr_fill_result = tr_matched_state.fill(&empty_content, true, &schema);
                                        if let Some(tr_needed_types) = tr_fill_result {
                                            println!("            tablerow 需要的类型数量: {}", tr_needed_types.len());
                                            for (k, type_name) in tr_needed_types.iter().enumerate() {
                                                println!("              需要的类型 {}: {}", k + 1, type_name);
                                            }
                                        } else {
                                            println!("            ❌ tablerow fill 返回 None");
                                        }
                                    } else {
                                        println!("            ❌ tablerow match_fragment 返回 None");
                                    }
                                } else {
                                    println!("            ❌ tablerow 没有 content_match");
                                }
                            } else {
                                println!("          ✅ tablerow 创建了 {} 个 tablecell", great_grandchildren.len());
                                for (k, ggc) in great_grandchildren.iter().enumerate() {
                                    let (ggc_node, _) = ggc.clone().into_parts();
                                    println!("            曾孙节点 {}: type={}, id={}", k + 1, ggc_node.r#type, ggc_node.id);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 对比：单独创建 table 节点
        println!("\n=== 对比：单独创建 Table 节点 ===");
        let table_type = schema.nodes.get("table").unwrap();
        let standalone_table = table_type.create_and_fill(
            None,
            None,
            vec![],
            None,
            &schema,
        );
        
        let (st_node, st_children) = standalone_table.into_parts();
        println!("单独的 Table 节点:");
        println!("  ID: {}", st_node.id);
        println!("  Content IDs: {:?}", st_node.content);
        println!("  子节点数量: {}", st_children.len());
        
        for (i, child) in st_children.iter().enumerate() {
            let (child_node, grandchildren) = child.clone().into_parts();
            println!("    子节点 {}: type={}, id={}", i + 1, child_node.r#type, child_node.id);
            println!("      孙节点数量: {}", grandchildren.len());
        }
        
        // 额外调试：单独创建 tablerow 节点
        println!("\n=== 额外调试：单独创建 tablerow 节点 ===");
        let tablerow_type = schema.nodes.get("tablerow").unwrap();
        let standalone_tablerow = tablerow_type.create_and_fill(
            None,
            None,
            vec![],
            None,
            &schema,
        );
        
        let (str_node, str_children) = standalone_tablerow.into_parts();
        println!("单独的 tablerow 节点:");
        println!("  ID: {}", str_node.id);
        println!("  Content IDs: {:?}", str_node.content);
        println!("  子节点数量: {}", str_children.len());
        
        for (i, child) in str_children.iter().enumerate() {
            let (child_node, _) = child.clone().into_parts();
            println!("    子节点 {}: type={}, id={}", i + 1, child_node.r#type, child_node.id);
        }
    }
}
