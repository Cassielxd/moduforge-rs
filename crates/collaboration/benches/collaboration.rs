use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mf_collab::*;
use std::collections::HashMap;
use serde_json::json;

/// 数据结构创建基准测试
fn bench_data_structures(c: &mut Criterion) {
    let mut group = c.benchmark_group("数据结构创建");

    // RoomSnapshot 创建
    group.bench_function("RoomSnapshot创建", |b| {
        b.iter(|| {
            let mut nodes = HashMap::new();
            nodes.insert(
                "node1".to_string(),
                NodeData {
                    id: "node1".to_string(),
                    node_type: "paragraph".to_string(),
                    attrs: HashMap::new(),
                    content: vec!["Hello World".to_string()],
                    marks: vec![],
                },
            );

            let snapshot = RoomSnapshot {
                room_id: "test_room".to_string(),
                root_id: "root".to_string(),
                nodes,
                version: 1,
            };
            criterion::black_box(snapshot)
        })
    });

    // NodeData 创建
    group.bench_function("NodeData创建", |b| {
        b.iter(|| {
            let node = NodeData {
                id: "test_node".to_string(),
                node_type: "paragraph".to_string(),
                attrs: {
                    let mut attrs = HashMap::new();
                    attrs.insert("class".to_string(), json!("content"));
                    attrs.insert("id".to_string(), json!("para1"));
                    attrs
                },
                content: vec!["Sample content".to_string()],
                marks: vec![MarkData {
                    mark_type: "bold".to_string(),
                    attrs: HashMap::new(),
                }],
            };
            criterion::black_box(node)
        })
    });

    // StepResult 创建
    group.bench_function("StepResult创建", |b| {
        b.iter(|| {
            let step_result = StepResult {
                step_id: "step_123".to_string(),
                step_name: "test_step".to_string(),
                description: "测试步骤".to_string(),
                timestamp: 1642000000000,
                client_id: "client_001".to_string(),
            };
            criterion::black_box(step_result)
        })
    });

    group.finish();
}

/// 批量数据处理基准测试
fn bench_batch_data_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("批量数据处理");

    // 批量NodeData创建
    for node_count in [10, 50, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("批量NodeData创建", node_count),
            node_count,
            |b, &count| {
                b.iter(|| {
                    let nodes: Vec<NodeData> = (0..count)
                        .map(|i| NodeData {
                            id: format!("node_{}", i),
                            node_type: "paragraph".to_string(),
                            attrs: {
                                let mut attrs = HashMap::new();
                                attrs.insert("index".to_string(), json!(i));
                                attrs
                            },
                            content: vec![format!("Content {}", i)],
                            marks: vec![],
                        })
                        .collect();
                    criterion::black_box(nodes)
                })
            },
        );
    }

    // 批量RoomSnapshot创建
    group.bench_function("批量RoomSnapshot", |b| {
        b.iter(|| {
            let snapshots: Vec<RoomSnapshot> = (0..20)
                .map(|i| {
                    let mut nodes = HashMap::new();
                    for j in 0..5 {
                        nodes.insert(
                            format!("node_{}_{}", i, j),
                            NodeData {
                                id: format!("node_{}_{}", i, j),
                                node_type: "paragraph".to_string(),
                                attrs: HashMap::new(),
                                content: vec![format!("Content {} {}", i, j)],
                                marks: vec![],
                            },
                        );
                    }

                    RoomSnapshot {
                        room_id: format!("room_{}", i),
                        root_id: "root".to_string(),
                        nodes,
                        version: 1,
                    }
                })
                .collect();
            criterion::black_box(snapshots)
        })
    });

    group.finish();
}

/// 序列化和反序列化基准测试
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("序列化");

    // NodeData 序列化
    group.bench_function("NodeData序列化", |b| {
        let node = NodeData {
            id: "test_node".to_string(),
            node_type: "paragraph".to_string(),
            attrs: {
                let mut attrs = HashMap::new();
                attrs.insert("class".to_string(), json!("content"));
                attrs
            },
            content: vec!["Test content".to_string()],
            marks: vec![],
        };

        b.iter(|| {
            let serialized = serde_json::to_string(&node).unwrap();
            criterion::black_box(serialized)
        })
    });

    // NodeData 反序列化
    group.bench_function("NodeData反序列化", |b| {
        let node_json = r#"{"id":"test_node","node_type":"paragraph","attrs":{"class":"content"},"content":["Test content"],"marks":[]}"#;

        b.iter(|| {
            let node: NodeData = serde_json::from_str(node_json).unwrap();
            criterion::black_box(node)
        })
    });

    // RoomSnapshot 序列化
    group.bench_function("RoomSnapshot序列化", |b| {
        let mut nodes = HashMap::new();
        nodes.insert(
            "node1".to_string(),
            NodeData {
                id: "node1".to_string(),
                node_type: "paragraph".to_string(),
                attrs: HashMap::new(),
                content: vec!["Content".to_string()],
                marks: vec![],
            },
        );

        let snapshot = RoomSnapshot {
            room_id: "test_room".to_string(),
            root_id: "root".to_string(),
            nodes,
            version: 1,
        };

        b.iter(|| {
            let serialized = serde_json::to_string(&snapshot).unwrap();
            criterion::black_box(serialized)
        })
    });

    group.finish();
}

/// RoomInfo 和状态管理基准测试
fn bench_room_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("房间管理");

    // RoomInfo 创建
    group.bench_function("RoomInfo创建", |b| {
        b.iter(|| {
            let room_info = RoomInfo {
                room_id: "test_room".to_string(),
                status: RoomStatus::Initialized,
                node_count: 100,
                client_count: 5,
                last_activity: std::time::SystemTime::now(),
            };
            criterion::black_box(room_info)
        })
    });

    // RoomStatus 状态转换
    group.bench_function("RoomStatus状态处理", |b| {
        let statuses = vec![
            RoomStatus::NotExists,
            RoomStatus::Created,
            RoomStatus::Initialized,
            RoomStatus::Shutting,
            RoomStatus::Offline,
        ];

        b.iter(|| {
            let processed_statuses: Vec<String> =
                statuses.iter().map(|status| format!("{:?}", status)).collect();
            criterion::black_box(processed_statuses)
        })
    });

    // 批量RoomInfo处理
    for room_count in [5, 20, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("批量RoomInfo", room_count),
            room_count,
            |b, &count| {
                b.iter(|| {
                    let rooms: Vec<RoomInfo> = (0..count)
                        .map(|i| RoomInfo {
                            room_id: format!("room_{}", i),
                            status: if i % 2 == 0 {
                                RoomStatus::Initialized
                            } else {
                                RoomStatus::Created
                            },
                            node_count: i * 10,
                            client_count: i % 5,
                            last_activity: std::time::SystemTime::now(),
                        })
                        .collect();
                    criterion::black_box(rooms)
                })
            },
        );
    }

    group.finish();
}

/// 复杂数据结构操作基准测试
fn bench_complex_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("复杂操作");

    // 复杂NodeData结构处理
    group.bench_function("复杂NodeData处理", |b| {
        b.iter(|| {
            let complex_node = NodeData {
                id: "complex_node".to_string(),
                node_type: "document".to_string(),
                attrs: {
                    let mut attrs = HashMap::new();
                    for i in 0..10 {
                        attrs.insert(
                            format!("attr_{}", i),
                            json!(format!("value_{}", i)),
                        );
                    }
                    attrs
                },
                content: (0..20)
                    .map(|i| format!("Content line {}", i))
                    .collect(),
                marks: (0..5)
                    .map(|i| MarkData {
                        mark_type: format!("mark_{}", i),
                        attrs: {
                            let mut mark_attrs = HashMap::new();
                            mark_attrs.insert(
                                "style".to_string(),
                                json!(format!("style_{}", i)),
                            );
                            mark_attrs
                        },
                    })
                    .collect(),
            };
            criterion::black_box(complex_node)
        })
    });

    // 大型RoomSnapshot处理
    group.bench_function("大型RoomSnapshot处理", |b| {
        b.iter(|| {
            let mut nodes = HashMap::new();

            // 创建大量节点
            for i in 0..100 {
                nodes.insert(
                    format!("node_{}", i),
                    NodeData {
                        id: format!("node_{}", i),
                        node_type: if i % 3 == 0 {
                            "heading"
                        } else {
                            "paragraph"
                        }
                        .to_string(),
                        attrs: {
                            let mut attrs = HashMap::new();
                            attrs.insert("index".to_string(), json!(i));
                            attrs.insert("level".to_string(), json!(i % 3 + 1));
                            attrs
                        },
                        content: vec![
                            format!("Large content for node {}", i).repeat(5),
                        ],
                        marks: if i % 5 == 0 {
                            vec![MarkData {
                                mark_type: "bold".to_string(),
                                attrs: HashMap::new(),
                            }]
                        } else {
                            vec![]
                        },
                    },
                );
            }

            let large_snapshot = RoomSnapshot {
                room_id: "large_room".to_string(),
                root_id: "root".to_string(),
                nodes,
                version: 100,
            };

            criterion::black_box(large_snapshot)
        })
    });

    group.finish();
}

/// 内存使用和性能基准测试
fn bench_memory_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("内存性能");

    // 内存密集型操作
    group.bench_function("内存密集型节点创建", |b| {
        b.iter(|| {
            // 创建大量小对象来测试内存分配性能
            let nodes: Vec<NodeData> = (0..1000)
                .map(|i| NodeData {
                    id: format!("mem_node_{}", i),
                    node_type: "text".to_string(),
                    attrs: HashMap::new(),
                    content: vec![i.to_string()],
                    marks: vec![],
                })
                .collect();

            // 计算总内容长度来避免优化掉计算
            let total_content: usize =
                nodes.iter().map(|n| n.content.len()).sum();

            criterion::black_box((nodes, total_content))
        })
    });

    // Clone性能测试
    group.bench_function("数据结构克隆", |b| {
        let original_node = NodeData {
            id: "original".to_string(),
            node_type: "paragraph".to_string(),
            attrs: {
                let mut attrs = HashMap::new();
                for i in 0..20 {
                    attrs.insert(
                        format!("key_{}", i),
                        json!(format!("value_{}", i)),
                    );
                }
                attrs
            },
            content: (0..50).map(|i| format!("Line {}", i)).collect(),
            marks: vec![
                MarkData {
                    mark_type: "bold".to_string(),
                    attrs: HashMap::new(),
                },
                MarkData {
                    mark_type: "italic".to_string(),
                    attrs: HashMap::new(),
                },
            ],
        };

        b.iter(|| {
            let cloned = original_node.clone();
            criterion::black_box(cloned)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_data_structures,
    bench_batch_data_processing,
    bench_serialization,
    bench_room_management,
    bench_complex_operations,
    bench_memory_performance
);
criterion_main!(benches);
