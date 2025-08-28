use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use mf_template::*;
use mf_expression::variable::Variable;
use std::rc::Rc;

/// 基础模板渲染基准测试
fn bench_basic_template_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("基础模板渲染");

    // 纯文本模板
    group.bench_function("纯文本模板", |b| {
        let template = "Hello World! This is a plain text template.";
        let context = Variable::Null;
        
        b.iter(|| {
            let result = render(template, context.clone());
            criterion::black_box(result)
        })
    });

    // 简单字符串上下文
    group.bench_function("简单字符串上下文", |b| {
        let template = "Hello World!";
        let context = Variable::String(Rc::from("test"));
        
        b.iter(|| {
            let result = render(template, context.clone());
            criterion::black_box(result)
        })
    });

    // Bool上下文
    group.bench_function("Bool上下文", |b| {
        let template = "Boolean value";
        let context = Variable::Bool(true);
        
        b.iter(|| {
            let result = render(template, context.clone());
            criterion::black_box(result)
        })
    });

    group.finish();
}

/// 模板解析基准测试
fn bench_template_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("模板解析");

    // 基础模板解析
    group.bench_function("基础模板解析", |b| {
        let template = "Hello World!";
        let context = Variable::Null;
        
        b.iter(|| {
            let result = render(template, context.clone());
            criterion::black_box(result)
        })
    });

    // 中等复杂度模板
    group.bench_function("中等复杂度模板", |b| {
        let template = "This is a medium complexity template with some text content.";
        let context = Variable::String(Rc::from("test"));
        
        b.iter(|| {
            let result = render(template, context.clone());
            criterion::black_box(result)
        })
    });

    // 复杂模板解析
    group.bench_function("复杂模板解析", |b| {
        let template = r"
        Welcome to ModuForge!
        
        This is a complex template with multiple lines
        and various content sections for testing
        performance characteristics.
        
        Template processing involves lexing, parsing,
        and rendering phases.
        
        Thank you for using our template engine!
        ";
        let context = Variable::String(Rc::from("complex"));
        
        b.iter(|| {
            let result = render(template, context.clone());
            criterion::black_box(result)
        })
    });

    group.finish();
}

/// 批量模板处理基准测试  
fn bench_batch_template_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("批量模板处理");

    // 批量简单模板
    for template_count in [10, 50, 200].iter() {
        group.bench_with_input(
            BenchmarkId::new("批量简单模板", template_count),
            template_count,
            |b, &count| {
                let templates: Vec<String> = (0..count)
                    .map(|i| format!("Template {}: Hello World!", i))
                    .collect();
                
                let contexts: Vec<Variable> = (0..count)
                    .map(|i| Variable::String(Rc::from(format!("Context{}", i))))
                    .collect();
                
                b.iter(|| {
                    let results: Vec<_> = templates.iter().zip(contexts.iter())
                        .map(|(template, context)| {
                            render(template, context.clone())
                        })
                        .collect();
                    criterion::black_box(results)
                })
            }
        );
    }

    // 批量复杂模板
    group.bench_function("批量复杂模板", |b| {
        let template = "Complex template with multiple sections and content areas.";
        let contexts: Vec<Variable> = (0..50)
            .map(|i| Variable::String(Rc::from(format!("Context{}", i))))
            .collect();
        
        b.iter(|| {
            let results: Vec<_> = contexts.iter()
                .map(|context| render(template, context.clone()))
                .collect();
            criterion::black_box(results)
        })
    });

    group.finish();
}

/// 错误处理基准测试
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("错误处理");

    // 正常处理
    group.bench_function("正常处理", |b| {
        let template = "Normal template processing";
        let context = Variable::String(Rc::from("test"));
        
        b.iter(|| {
            let result = render(template, context.clone());
            criterion::black_box(result)
        })
    });

    // 空模板处理
    group.bench_function("空模板处理", |b| {
        let template = "";
        let context = Variable::Null;
        
        b.iter(|| {
            let result = render(template, context.clone());
            criterion::black_box(result)
        })
    });

    // Null上下文处理
    group.bench_function("Null上下文处理", |b| {
        let template = "Template with null context";
        let context = Variable::Null;
        
        b.iter(|| {
            let result = render(template, context.clone());
            criterion::black_box(result)
        })
    });

    group.finish();
}

/// 性能压力测试
fn bench_performance_stress_tests(c: &mut Criterion) {
    let mut group = c.benchmark_group("性能压力测试");

    // 长模板处理
    group.bench_function("长模板处理", |b| {
        let template_parts = (0..100)
            .map(|i| format!("Section {}: This is content section with text data. ", i))
            .collect::<Vec<_>>()
            .join(" ");
        
        let context = Variable::String(Rc::from("long_content"));
        
        b.iter(|| {
            let result = render(&template_parts, context.clone());
            criterion::black_box(result)
        })
    });

    // 频繁模板处理
    group.bench_function("频繁模板处理", |b| {
        let template = "Frequent processing template";
        
        b.iter(|| {
            let mut results = Vec::new();
            for i in 0..50 {
                let context = Variable::String(Rc::from(format!("Context{}", i)));
                let result = render(template, context);
                results.push(result);
            }
            criterion::black_box(results)
        })
    });

    group.finish();
}

/// 内存使用基准测试
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("内存使用");

    // 模板重用
    group.bench_function("模板重用", |b| {
        let template = "Reusable template for testing memory usage patterns.";
        
        b.iter(|| {
            let mut results = Vec::new();
            for i in 0..100 {
                let context = Variable::String(Rc::from(format!("User{}", i)));
                let result = render(template, context);
                results.push(result);
            }
            criterion::black_box(results)
        })
    });

    // 上下文克隆性能
    group.bench_function("上下文克隆", |b| {
        let large_context = Variable::String(Rc::from("Large context content".repeat(100)));
        
        b.iter(|| {
            let cloned = large_context.clone();
            criterion::black_box(cloned)
        })
    });

    // 大量小模板
    group.bench_function("大量小模板", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for i in 0..500 {
                let template = format!("Item {}: content", i);
                let context = Variable::String(Rc::from(format!("Value{}", i)));
                
                let result = render(&template, context);
                results.push(result);
            }
            criterion::black_box(results)
        })
    });

    group.finish();
}

/// 数据类型处理基准测试
fn bench_data_type_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("数据类型处理");

    // String类型处理
    group.bench_function("String类型处理", |b| {
        let template = "String template";
        let contexts: Vec<Variable> = (0..50)
            .map(|i| Variable::String(Rc::from(format!("String{}", i))))
            .collect();
        
        b.iter(|| {
            let results: Vec<_> = contexts.iter()
                .map(|context| render(template, context.clone()))
                .collect();
            criterion::black_box(results)
        })
    });

    // Bool类型处理
    group.bench_function("Bool类型处理", |b| {
        let template = "Boolean template";
        let contexts: Vec<Variable> = (0..50)
            .map(|i| Variable::Bool(i % 2 == 0))
            .collect();
        
        b.iter(|| {
            let results: Vec<_> = contexts.iter()
                .map(|context| render(template, context.clone()))
                .collect();
            criterion::black_box(results)
        })
    });

    // Null类型处理
    group.bench_function("Null类型处理", |b| {
        let template = "Null template";
        let context = Variable::Null;
        
        b.iter(|| {
            let result = render(template, context.clone());
            criterion::black_box(result)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_basic_template_rendering,
    bench_template_parsing,
    bench_batch_template_processing,
    bench_error_handling,
    bench_performance_stress_tests,
    bench_memory_usage,
    bench_data_type_handling
);
criterion_main!(benches);