use crate::model::IndexDoc;

/// 索引增量变更（后端消费的基本指令）
#[derive(Debug, Clone)]
pub enum IndexMutation {
    Add(IndexDoc),
    Upsert(IndexDoc),
    DeleteById(String),
    DeleteManyById(Vec<String>),
}

/// 简单查询条件
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    /// 全文查询（走 text 字段）
    pub text: Option<String>,
    /// 节点类型精确匹配
    pub node_type: Option<String>,
    /// 父节点精确匹配
    pub parent_id: Option<String>,
    /// 标记（所有给定的标记都需存在）
    pub marks: Vec<String>,
    /// 扁平属性匹配（k=v）
    pub attrs: Vec<(String, String)>,
    /// 返回条数限制（默认 50）
    pub limit: usize,
    /// 偏移量
    pub offset: usize,
    /// 排序字段（fast field 名称），仅支持 i64 类型
    pub sort_by: Option<String>,
    /// 排序方向 true=升序，false=降序
    pub sort_asc: bool,
    /// search-after 游标：上一页最后一个文档的排序值（i64），与 sort_by 搭配使用
    pub after_value: Option<i64>,
    /// 可选的范围过滤：fast field i64 [min, max]
    pub range_field: Option<String>,
    pub range_min: Option<i64>,
    pub range_max: Option<i64>,
}

// 不再使用通用接口，直接以 TantivyBackend 作为唯一后端

// ------------------ Tantivy 后端 ------------------
use tantivy::{schema::Schema, Index, Term};
use tantivy::schema::document::Value as _;
use parking_lot::Mutex;
use std::path::PathBuf;
use tantivy_jieba::JiebaTokenizer;
use tantivy::IndexReader;

/// Tantivy 后端实现
pub struct TantivyBackend {
    index: Index,
    schema: Schema,
    writer: Mutex<tantivy::IndexWriter>,
    reader: IndexReader,
    fields: TantivyFields,
    /// 若使用临时目录创建，保存 TempDir 以便生命周期结束后自动清理
    #[allow(dead_code)]
    temp_guard: Option<tempfile::TempDir>,
    /// 索引目录绝对路径（便于多实例时区分）
    index_dir: PathBuf,
}

#[derive(Clone, Copy)]
struct TantivyFields {
    node_id: tantivy::schema::Field,
    node_type: tantivy::schema::Field,
    parent_id: tantivy::schema::Field,
    path_facet: tantivy::schema::Field,
    marks: tantivy::schema::Field,
    attrs_flat: tantivy::schema::Field,
    text: tantivy::schema::Field,
    // fast fields (i64)
    order_i64: tantivy::schema::Field,
    created_at_i64: tantivy::schema::Field,
    updated_at_i64: tantivy::schema::Field,
}

impl TantivyBackend {
    pub fn new_in_dir(dir: &std::path::Path) -> anyhow::Result<Self> {
        let (schema, fields) = build_schema();
        let index = if dir.exists() {
            Index::open_in_dir(dir)?
        } else {
            std::fs::create_dir_all(dir)?;
            Index::create_in_dir(dir, schema.clone())?
        };
        // 注册 jieba 中文分词器
        index.tokenizers().register("jieba_zh", JiebaTokenizer {});
        let writer = index.writer(128_000_000)?; // 128MB 写缓冲
        let reader: IndexReader = index.reader_builder().try_into()?;
        Ok(Self {
            index,
            schema,
            writer: Mutex::new(writer),
            reader,
            fields,
            temp_guard: None,
            index_dir: dir.canonicalize().unwrap_or(dir.to_path_buf()),
        })
    }

    /// 在指定临时根目录下创建唯一索引目录并初始化
    /// 例如传入 `C:/tmp/mf_index`，将创建 `C:/tmp/mf_index/<随机>/`
    pub fn new_in_temp_root(
        temp_root: &std::path::Path
    ) -> anyhow::Result<Self> {
        let td = tempfile::Builder::new()
            .prefix("mf_index_")
            .tempdir_in(temp_root)?;
        let dir = td.path().to_path_buf();
        let (schema, fields) = build_schema();
        let index = Index::create_in_dir(&dir, schema.clone())?;
        index.tokenizers().register("jieba_zh", JiebaTokenizer {});
        let writer = index.writer(128_000_000)?;
        let reader: IndexReader = index.reader_builder().try_into()?;
        Ok(Self {
            index,
            schema,
            writer: Mutex::new(writer),
            reader,
            fields,
            temp_guard: Some(td),
            index_dir: dir,
        })
    }

    /// 使用系统临时目录创建索引
    pub fn new_in_system_temp() -> anyhow::Result<Self> {
        let td = tempfile::Builder::new().prefix("mf_index_").tempdir()?;
        let dir = td.path().to_path_buf();
        let (schema, fields) = build_schema();
        let index = Index::create_in_dir(&dir, schema.clone())?;
        index.tokenizers().register("jieba_zh", JiebaTokenizer {});
        let writer = index.writer(128_000_000)?;
        let reader: IndexReader = index.reader_builder().try_into()?;
        Ok(Self {
            index,
            schema,
            writer: Mutex::new(writer),
            reader,
            fields,
            temp_guard: Some(td),
            index_dir: dir,
        })
    }

    /// 获取索引目录路径
    pub fn index_dir(&self) -> &std::path::Path {
        &self.index_dir
    }
}

fn build_schema() -> (Schema, TantivyFields) {
    use tantivy::schema::*;
    let mut builder = Schema::builder();
    let node_id = builder.add_text_field("node_id", STRING | STORED);
    let node_type = builder.add_text_field("node_type", STRING);
    let parent_id = builder.add_text_field("parent_id", STRING);
    let path_facet =
        builder.add_facet_field("path_facet", FacetOptions::default());
    let marks = builder.add_text_field("marks", STRING);
    let attrs_flat = builder.add_text_field("attrs_flat", STRING);
    // 注册中文分词器（jieba）为 "jieba_zh"，并用于 text 字段
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("jieba_zh")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_opts = TextOptions::default().set_indexing_options(text_indexing);
    let text = builder.add_text_field("text", text_opts);
    // fast fields
    let order_i64 = builder.add_i64_field("order_i64", FAST);
    let created_at_i64 = builder.add_i64_field("created_at_i64", FAST);
    let updated_at_i64 = builder.add_i64_field("updated_at_i64", FAST);
    let schema = builder.build();
    // 注意：schema 构建完成后再注册分词器到 Index
    (
        schema,
        TantivyFields {
            node_id,
            node_type,
            parent_id,
            path_facet,
            marks,
            attrs_flat,
            text,
            order_i64,
            created_at_i64,
            updated_at_i64,
        },
    )
}

fn add_index_doc(
    writer: &mut tantivy::IndexWriter,
    _schema: &Schema,
    fields: TantivyFields,
    nd: IndexDoc,
) -> anyhow::Result<()> {
    use tantivy::schema::{Facet, TantivyDocument};
    let mut doc = TantivyDocument::default();
    doc.add_text(fields.node_id, &nd.node_id);
    doc.add_text(fields.node_type, &nd.node_type);
    if let Some(pid) = &nd.parent_id {
        doc.add_text(fields.parent_id, pid);
    }
    // 路径 Facet："/a/b/c"
    let facet_path = format!("/{}", nd.path.join("/"));
    doc.add_facet(fields.path_facet, Facet::from(facet_path.as_str()));
    for m in &nd.marks {
        doc.add_text(fields.marks, m);
    }
    for (k, v) in &nd.attrs_flat {
        doc.add_text(fields.attrs_flat, format!("{k}={v}"));
    }
    if let Some(t) = &nd.text {
        doc.add_text(fields.text, t);
    }
    // fast fields
    if let Some(v) = nd.order_i64 {
        doc.add_i64(fields.order_i64, v);
    }
    if let Some(v) = nd.created_at_i64 {
        doc.add_i64(fields.created_at_i64, v);
    }
    if let Some(v) = nd.updated_at_i64 {
        doc.add_i64(fields.updated_at_i64, v);
    }
    writer.add_document(doc)?;
    Ok(())
}

impl TantivyBackend {
    pub async fn apply(
        &self,
        mutations: Vec<IndexMutation>,
    ) -> anyhow::Result<()> {
        let mut writer = self.writer.lock();
        for m in mutations {
            match m {
                IndexMutation::Add(d) | IndexMutation::Upsert(d) => {
                    // Upsert as delete+add
                    writer.delete_term(Term::from_field_text(
                        self.fields.node_id,
                        &d.node_id,
                    ));
                    add_index_doc(&mut writer, &self.schema, self.fields, d)?;
                },
                IndexMutation::DeleteById(id) => {
                    writer.delete_term(Term::from_field_text(
                        self.fields.node_id,
                        &id,
                    ));
                },
                IndexMutation::DeleteManyById(ids) => {
                    for id in ids {
                        writer.delete_term(Term::from_field_text(
                            self.fields.node_id,
                            &id,
                        ));
                    }
                },
            }
        }
        writer.commit()?;
        // 使新提交对查询可见
        self.reader.reload()?;
        Ok(())
    }

    pub async fn rebuild_all(
        &self,
        docs: Vec<IndexDoc>,
    ) -> anyhow::Result<()> {
        // 简单实现：删除已有文档的 node_id，再批量写入新文档
        let mut writer = self.writer.lock();
        // tantivy 不支持全清，这里逐个删除，后续 upsert
        for d in &docs {
            writer.delete_term(Term::from_field_text(
                self.fields.node_id,
                &d.node_id,
            ));
        }
        for d in docs {
            add_index_doc(&mut writer, &self.schema, self.fields, d)?;
        }
        writer.commit()?;
        // 使新提交对查询可见
        self.reader.reload()?;
        Ok(())
    }

    pub async fn search_ids(
        &self,
        query: SearchQuery,
    ) -> anyhow::Result<Vec<String>> {
        use tantivy::collector::TopDocs;
        use tantivy::query::{BooleanQuery, Occur, Query, TermQuery};
        use tantivy::Order;
        use tantivy::query::RangeQuery;
        use std::ops::Bound;
        let searcher = self.reader.searcher();

        let mut subqueries: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        if let Some(t) = &query.node_type {
            let term = Term::from_field_text(self.fields.node_type, t);
            subqueries.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    term,
                    tantivy::schema::IndexRecordOption::Basic,
                )),
            ));
        }
        if let Some(p) = &query.parent_id {
            let term = Term::from_field_text(self.fields.parent_id, p);
            subqueries.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    term,
                    tantivy::schema::IndexRecordOption::Basic,
                )),
            ));
        }
        for m in &query.marks {
            let term = Term::from_field_text(self.fields.marks, m);
            subqueries.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    term,
                    tantivy::schema::IndexRecordOption::Basic,
                )),
            ));
        }
        for (k, v) in &query.attrs {
            // match on flattened k=v string
            let kv = format!("{k}={v}");
            let term = Term::from_field_text(self.fields.attrs_flat, &kv);
            subqueries.push((
                Occur::Must,
                Box::new(TermQuery::new(
                    term,
                    tantivy::schema::IndexRecordOption::Basic,
                )),
            ));
        }

        // 文本查询（QueryParser）
        if let Some(q) = &query.text {
            let qp = tantivy::query::QueryParser::for_index(
                &self.index,
                vec![self.fields.text],
            );
            if let Ok(qx) = qp.parse_query(q) {
                subqueries.push((Occur::Must, qx));
            }
        }

        // 范围过滤（fast field i64）
        if let Some(field_name) = &query.range_field {
            let field = match field_name.as_str() {
                "order_i64" => Some(self.fields.order_i64),
                "created_at_i64" => Some(self.fields.created_at_i64),
                "updated_at_i64" => Some(self.fields.updated_at_i64),
                _ => None,
            };
            if let Some(f) = field {
                let lower = match query.range_min {
                    Some(v) => Bound::Included(Term::from_field_i64(f, v)),
                    None => Bound::Unbounded,
                };
                let upper = match query.range_max {
                    Some(v) => Bound::Included(Term::from_field_i64(f, v)),
                    None => Bound::Unbounded,
                };
                let rq = RangeQuery::new(lower, upper);
                subqueries.push((Occur::Must, Box::new(rq)));
            }
        }

        // search-after（基于排序字段的边界）
        if let (Some(sort_by), Some(after_val)) =
            (&query.sort_by, query.after_value)
        {
            let (field_opt, lower_upper): (
                Option<tantivy::schema::Field>,
                (Bound<Term>, Bound<Term>),
            ) = match sort_by.as_str() {
                "order_i64" => (
                    Some(self.fields.order_i64),
                    if query.sort_asc {
                        (
                            Bound::Excluded(Term::from_field_i64(
                                self.fields.order_i64,
                                after_val,
                            )),
                            Bound::Unbounded,
                        )
                    } else {
                        (
                            Bound::Unbounded,
                            Bound::Excluded(Term::from_field_i64(
                                self.fields.order_i64,
                                after_val,
                            )),
                        )
                    },
                ),
                "created_at_i64" => (
                    Some(self.fields.created_at_i64),
                    if query.sort_asc {
                        (
                            Bound::Excluded(Term::from_field_i64(
                                self.fields.created_at_i64,
                                after_val,
                            )),
                            Bound::Unbounded,
                        )
                    } else {
                        (
                            Bound::Unbounded,
                            Bound::Excluded(Term::from_field_i64(
                                self.fields.created_at_i64,
                                after_val,
                            )),
                        )
                    },
                ),
                "updated_at_i64" => (
                    Some(self.fields.updated_at_i64),
                    if query.sort_asc {
                        (
                            Bound::Excluded(Term::from_field_i64(
                                self.fields.updated_at_i64,
                                after_val,
                            )),
                            Bound::Unbounded,
                        )
                    } else {
                        (
                            Bound::Unbounded,
                            Bound::Excluded(Term::from_field_i64(
                                self.fields.updated_at_i64,
                                after_val,
                            )),
                        )
                    },
                ),
                _ => (None, (Bound::Unbounded, Bound::Unbounded)),
            };
            if let Some(_f) = field_opt {
                let rq = RangeQuery::new(lower_upper.0, lower_upper.1);
                subqueries.push((Occur::Must, Box::new(rq)));
            }
        }

        let boxed_query: Box<dyn Query> = if subqueries.is_empty() {
            Box::new(tantivy::query::AllQuery)
        } else {
            Box::new(BooleanQuery::new(subqueries))
        };

        // 基于 fast field 排序，并支持 search-after（通过上面的 RangeQuery 约束实现）
        let limit = if query.limit == 0 { 50 } else { query.limit };
        let addrs: Vec<tantivy::DocAddress> = if let Some(sort_by) =
            &query.sort_by
        {
            let order = if query.sort_asc { Order::Asc } else { Order::Desc };
            match sort_by.as_str() {
                "order_i64" => {
                    let collector = TopDocs::with_limit(limit)
                        .order_by_fast_field::<i64>("order_i64", order);
                    let sorted: Vec<(i64, tantivy::DocAddress)> =
                        searcher.search(&*boxed_query, &collector)?;
                    sorted.into_iter().map(|(_, addr)| addr).collect()
                },
                "created_at_i64" => {
                    let collector = TopDocs::with_limit(limit)
                        .order_by_fast_field::<i64>("created_at_i64", order);
                    let sorted: Vec<(i64, tantivy::DocAddress)> =
                        searcher.search(&*boxed_query, &collector)?;
                    sorted.into_iter().map(|(_, addr)| addr).collect()
                },
                "updated_at_i64" => {
                    let collector = TopDocs::with_limit(limit)
                        .order_by_fast_field::<i64>("updated_at_i64", order);
                    let sorted: Vec<(i64, tantivy::DocAddress)> =
                        searcher.search(&*boxed_query, &collector)?;
                    sorted.into_iter().map(|(_, addr)| addr).collect()
                },
                _ => {
                    let top_docs: Vec<(f32, tantivy::DocAddress)> = searcher
                        .search(&*boxed_query, &TopDocs::with_limit(limit))?;
                    top_docs.into_iter().map(|(_, addr)| addr).collect()
                },
            }
        } else {
            let top_docs: Vec<(f32, tantivy::DocAddress)> =
                searcher.search(&*boxed_query, &TopDocs::with_limit(limit))?;
            top_docs.into_iter().map(|(_, addr)| addr).collect()
        };
        let mut ids = Vec::new();
        for addr in addrs.into_iter().skip(query.offset) {
            let doc: tantivy::schema::TantivyDocument = searcher.doc(addr)?;
            if let Some(val) = doc.get_first(self.fields.node_id) {
                if let tantivy::schema::document::ReferenceValue::Leaf(
                    tantivy::schema::document::ReferenceValueLeaf::Str(s),
                ) = val.as_value()
                {
                    ids.push(s.to_string())
                }
            }
        }
        Ok(ids)
    }
}
