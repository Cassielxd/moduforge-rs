use std::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use std::cmp::Ordering;

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
        ContentMatch { next: Vec::new(), wrap_cache: Vec::new(), valid_end: true }
    }

    pub fn match_type(
        &self,
        node_type: &NodeType,
    ) -> Option<&ContentMatch> {
        self.next.iter().find(|edge| &edge.node_type == node_type).map(|edge| &edge.next)
    }

    pub fn match_fragment(
        &self,
        frag: &[Node],
        schema: &Schema,
    ) -> Option<&ContentMatch> {
        let mut current: &ContentMatch = self;

        for content in frag.iter() {
            if let Some(next) = current.match_type(schema.nodes.get(&content.r#type).unwrap()) {
                current = next;
            }
        }
        Some(current)
    }

    pub fn fill(
        &self,
        after: &Vec<Node>,
        to_end: bool,
        schema: &Schema,
    ) -> Option<Vec<Node>> {
        let mut seen: Vec<ContentMatch> = Vec::new();
        seen.push(self.clone());
        fn search(
            seen: &mut Vec<ContentMatch>,
            to_end: bool,
            after: &Vec<Node>,
            match_: &ContentMatch,
            types: &mut Vec<NodeType>,
            schema: &Schema,
        ) -> Option<Vec<Node>> {
            if let Some(finished) = match_.match_fragment(after, schema) {
                if finished.valid_end || !to_end {
                    let mut nodes = vec![];
                    for tp in types.iter() {
                        let mut node = tp.create_and_fill(None, None, vec![], None, schema);
                        nodes.append(&mut node);
                    }
                    return Some(nodes);
                }
            }
            for edge in &match_.next {
                if !edge.node_type.has_required_attrs() && !seen.contains(&edge.next) {
                    seen.push(edge.next.clone());
                    types.push(edge.node_type.clone());
                    if let Some(found) = search(seen, to_end, after, &edge.next, types, schema) {
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
        self.next.iter().find(|edge| !edge.node_type.has_required_attrs()).map(|edge| &edge.node_type)
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
    ) -> Result<&MatchEdge, String> {
        if n >= self.next.len() { Err(format!("{} 超出了 {}", n, self.next.len())) } else { Ok(&self.next[n]) }
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
                let mut out = format!("{} ", if m.valid_end { i + 1 } else { i });
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
        stream.err(&format!("No node type or group '{}' found", name));
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
            let exprs: Vec<Expr> =
                resolve_name(stream, next).into_iter().map(|type_| Expr::Name { value: type_ }).collect();
            stream.pos += 1;
            if exprs.len() == 1 { exprs.into_iter().next().unwrap() } else { Expr::Choice { exprs } }
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
        let mut state =
            ContentMatch { next: Vec::new(), wrap_cache: vec![], valid_end: states.contains(&(nfa.len() - 1)) };

        let state_key = states.iter().map(|&x| x.to_string()).collect::<Vec<_>>().join(",");
        labeled.insert(state_key.clone(), state.clone());

        for (term, states) in out {
            let states_key = states.iter().map(|&x| x.to_string()).collect::<Vec<_>>().join(",");
            let next_state = labeled.get(&states_key).cloned().unwrap_or_else(|| explore(states, nfa, labeled));
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
    let edge = Rc::new(RefCell::new(Edge { term, to: Option::from(to.unwrap_or(0)) }));
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
        Expr::Choice { exprs } => exprs.into_iter().flat_map(|expr| compile(expr, from, nfa)).collect(),
        Expr::Seq { exprs } => {
            let mut cur = from;
            let len = exprs.len(); // 在这里获取长度
            for (i, expr) in exprs.into_iter().enumerate() {
                let mut next = compile(expr, cur, nfa);
                if i == len - 1 {
                    return next;
                }
                connect(&mut next, node(nfa));
                cur = node(nfa);
            }
            vec![]
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
