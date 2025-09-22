use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mf_macro::*;

/// 基础宏展开基准测试
fn bench_basic_macro_expansion(c: &mut Criterion) {
    let mut group = c.benchmark_group("基础宏展开");

    // node! 宏基础使用
    group.bench_function("node!宏基础展开", |b| {
        b.iter(|| {
            let node = node!("test_node");
            criterion::black_box(node)
        })
    });

    // node! 宏带描述
    group.bench_function("node!宏带描述", |b| {
        b.iter(|| {
            let node = node!("test_node", "这是一个测试节点");
            criterion::black_box(node)
        })
    });

    // node! 宏带内容
    group.bench_function("node!宏带内容", |b| {
        b.iter(|| {
            let node = node!("test_node", "测试节点", "节点内容");
            criterion::black_box(node)
        })
    });

    group.finish();
}

/// mark宏展开基准测试
fn bench_mark_macro_expansion(c: &mut Criterion) {
    let mut group = c.benchmark_group("mark宏展开");

    // mark! 宏基础使用
    group.bench_function("mark!宏基础展开", |b| {
        b.iter(|| {
            let mark = mark!("bold");
            criterion::black_box(mark)
        })
    });

    // mark! 宏带描述
    group.bench_function("mark!宏带描述", |b| {
        b.iter(|| {
            let mark = mark!("bold", "粗体标记");
            criterion::black_box(mark)
        })
    });

    group.finish();
}

/// 批量宏使用基准测试
fn bench_batch_macro_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("批量宏使用");

    // 批量node创建
    for node_count in [10, 50, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("批量node!创建", node_count),
            node_count,
            |b, &count| {
                b.iter(|| {
                    let nodes: Vec<_> = (0..count)
                        .map(|i| {
                            node!(
                                &format!("node_{}", i),
                                &format!("节点 {}", i),
                                &format!("内容 {}", i)
                            )
                        })
                        .collect();
                    criterion::black_box(nodes)
                })
            },
        );
    }

    // 批量mark创建
    for mark_count in [5, 20, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("批量mark!创建", mark_count),
            mark_count,
            |b, &count| {
                b.iter(|| {
                    let marks: Vec<_> = (0..count)
                        .map(|i| {
                            mark!(
                                &format!("mark_{}", i),
                                &format!("标记 {}", i)
                            )
                        })
                        .collect();
                    criterion::black_box(marks)
                })
            },
        );
    }

    group.finish();
}

/// 复杂宏组合基准测试
fn bench_complex_macro_combinations(c: &mut Criterion) {
    let mut group = c.benchmark_group("复杂宏组合");

    // 复杂node结构
    group.bench_function("复杂node结构创建", |b| {
        b.iter(|| {
            let document = node!("document", "文档根节点", "文档内容");

            let section = node!("section", "章节节点", "章节内容");

            let paragraph = node!("paragraph", "段落节点", "段落文本内容");

            criterion::black_box((document, section, paragraph))
        })
    });

    // 复杂mark组合
    group.bench_function("复杂mark组合", |b| {
        b.iter(|| {
            let bold = mark!("bold", "粗体标记");
            let italic = mark!("italic", "斜体标记");
            let underline = mark!("underline", "下划线标记");
            let highlight = mark!("highlight", "高亮标记");

            criterion::black_box((bold, italic, underline, highlight))
        })
    });

    // 嵌套结构创建
    group.bench_function("嵌套结构创建", |b| {
        b.iter(|| {
            let mut nodes = Vec::new();
            let mut marks = Vec::new();

            // 创建嵌套的node结构
            for i in 0..5 {
                let section = node!(
                    &format!("section_{}", i),
                    &format!("章节 {}", i + 1),
                    &format!("章节 {} 的内容", i + 1)
                );
                nodes.push(section);

                // 为每个section创建对应的mark
                for j in 0..3 {
                    let mark = mark!(
                        &format!("mark_{}_{}", i, j),
                        &format!("标记 {}.{}", i + 1, j + 1)
                    );
                    marks.push(mark);
                }
            }

            criterion::black_box((nodes, marks))
        })
    });

    group.finish();
}

/// 宏编译时性能基准测试
fn bench_macro_compile_time_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("宏编译时性能");

    // 不同复杂度的node宏
    group.bench_function("简单node宏", |b| {
        b.iter(|| {
            let node = node!("simple");
            criterion::black_box(node)
        })
    });

    group.bench_function("中等复杂度node宏", |b| {
        b.iter(|| {
            let node = node!("medium", "中等复杂度节点", "节点内容");
            criterion::black_box(node)
        })
    });

    group.bench_function("高复杂度node宏", |b| {
        b.iter(|| {
            let node = node!(
                "complex",
                "高复杂度节点包含大量属性和长描述文本用于测试宏展开性能",
                "这是一个包含大量内容文本的节点，用于测试node宏在处理长文本时的性能表现"
            );
            criterion::black_box(node)
        })
    });

    group.finish();
}

/// 内存使用基准测试
fn bench_macro_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("宏内存使用");

    // 大量宏实例创建
    group.bench_function("大量node实例", |b| {
        b.iter(|| {
            let nodes: Vec<_> = (0..1000)
                .map(|i| {
                    node!(
                        &format!("mass_node_{}", i),
                        &format!("批量节点 {}", i),
                        &format!("节点内容 {}", i)
                    )
                })
                .collect();
            criterion::black_box(nodes)
        })
    });

    // 复杂宏实例创建
    group.bench_function("复杂宏实例", |b| {
        b.iter(|| {
            let complex_nodes: Vec<_> = (0..100)
                .map(|i| {
                    node!(
                        &format!("complex_node_{}", i),
                        &format!("复杂节点 {} 包含大量信息和元数据", i),
                        &format!("这是节点 {} 的详细内容，包含多种类型的数据和属性信息", i)
                    )
                })
                .collect();
            criterion::black_box(complex_nodes)
        })
    });

    // Clone性能
    group.bench_function("宏创建对象克隆", |b| {
        let original = node!(
            "original_node",
            "原始节点用于克隆测试",
            "这是原始节点的内容，将被克隆多次"
        );

        b.iter(|| {
            let cloned = original.clone();
            criterion::black_box(cloned)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_basic_macro_expansion,
    bench_mark_macro_expansion,
    bench_batch_macro_usage,
    bench_complex_macro_combinations,
    bench_macro_compile_time_performance,
    bench_macro_memory_usage
);
criterion_main!(benches);
