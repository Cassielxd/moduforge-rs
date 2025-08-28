#!/usr/bin/env python3
"""
生成综合性能报告HTML页面

用法:
python3 generate_comprehensive_report.py --input results.json --output report.html --title "性能报告" --commit abc123
"""

import json
import argparse
from datetime import datetime
from typing import Dict, List
import statistics

HTML_TEMPLATE = """
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif;
            background-color: #f5f5f7;
            color: #1d1d1f;
            line-height: 1.6;
        }}
        
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 2rem;
            text-align: center;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        
        .header h1 {{
            font-size: 2.5rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
        }}
        
        .header .subtitle {{
            opacity: 0.9;
            font-size: 1.1rem;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            padding: 2rem;
        }}
        
        .summary-cards {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 1.5rem;
            margin-bottom: 3rem;
        }}
        
        .card {{
            background: white;
            border-radius: 12px;
            padding: 1.5rem;
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
            transition: transform 0.2s ease;
        }}
        
        .card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 8px 25px rgba(0,0,0,0.15);
        }}
        
        .card-title {{
            font-size: 0.9rem;
            color: #666;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            margin-bottom: 0.5rem;
        }}
        
        .card-value {{
            font-size: 2rem;
            font-weight: 700;
            color: #333;
        }}
        
        .chart-container {{
            background: white;
            border-radius: 12px;
            padding: 2rem;
            margin-bottom: 2rem;
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        }}
        
        .chart-title {{
            font-size: 1.3rem;
            font-weight: 600;
            margin-bottom: 1rem;
            color: #333;
        }}
        
        .tier-section {{
            background: white;
            border-radius: 12px;
            padding: 2rem;
            margin-bottom: 2rem;
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        }}
        
        .tier-title {{
            font-size: 1.5rem;
            font-weight: 600;
            margin-bottom: 1rem;
            padding-bottom: 0.5rem;
            border-bottom: 2px solid #f0f0f0;
        }}
        
        .crate-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 1rem;
            margin-top: 1rem;
        }}
        
        .crate-card {{
            background: #f8f9fa;
            border-radius: 8px;
            padding: 1rem;
            border-left: 4px solid #667eea;
        }}
        
        .crate-name {{
            font-weight: 600;
            color: #333;
            margin-bottom: 0.5rem;
        }}
        
        .benchmark-list {{
            list-style: none;
        }}
        
        .benchmark-item {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 0.25rem 0;
            font-size: 0.9rem;
        }}
        
        .benchmark-name {{
            color: #666;
        }}
        
        .benchmark-time {{
            font-family: 'Monaco', 'Consolas', monospace;
            color: #333;
            font-weight: 500;
        }}
        
        .footer {{
            text-align: center;
            padding: 2rem;
            color: #666;
            border-top: 1px solid #e0e0e0;
            margin-top: 3rem;
        }}
        
        .performance-badge {{
            display: inline-block;
            padding: 0.25rem 0.75rem;
            border-radius: 20px;
            font-size: 0.8rem;
            font-weight: 600;
        }}
        
        .badge-excellent {{ background: #e6ffe6; color: #2d5016; }}
        .badge-good {{ background: #fffbe6; color: #594d00; }}
        .badge-warning {{ background: #ffe6e6; color: #5a1a1a; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>{title}</h1>
        <div class="subtitle">
            提交: {commit} | 生成时间: {timestamp}
        </div>
    </div>
    
    <div class="container">
        <!-- 汇总卡片 -->
        <div class="summary-cards">
            <div class="card">
                <div class="card-title">总基准测试数</div>
                <div class="card-value">{total_benchmarks}</div>
            </div>
            <div class="card">
                <div class="card-title">测试的Crate数</div>
                <div class="card-value">{total_crates}</div>
            </div>
            <div class="card">
                <div class="card-title">平均执行时间</div>
                <div class="card-value">{avg_duration}</div>
            </div>
            <div class="card">
                <div class="card-title">总执行层级</div>
                <div class="card-value">{total_tiers}</div>
            </div>
        </div>
        
        <!-- 性能分布图表 -->
        <div class="chart-container">
            <div class="chart-title">📊 执行时间分布</div>
            <canvas id="durationChart" width="400" height="200"></canvas>
        </div>
        
        <div class="chart-container">
            <div class="chart-title">🏗️ 各层级性能对比</div>
            <canvas id="tierChart" width="400" height="200"></canvas>
        </div>
        
        <!-- 分层级详细结果 -->
        {tier_sections}
        
        <div class="footer">
            <p>ModuForge-RS 性能基准测试报告</p>
            <p>由 GitHub Actions 自动生成</p>
        </div>
    </div>
    
    <script>
        // 执行时间分布图表
        const durationCtx = document.getElementById('durationChart').getContext('2d');
        new Chart(durationCtx, {{
            type: 'bar',
            data: {{
                labels: {crate_names},
                datasets: [{{
                    label: '平均执行时间 (毫秒)',
                    data: {duration_data},
                    backgroundColor: 'rgba(102, 126, 234, 0.6)',
                    borderColor: 'rgba(102, 126, 234, 1)',
                    borderWidth: 1
                }}]
            }},
            options: {{
                responsive: true,
                scales: {{
                    y: {{
                        beginAtZero: true,
                        title: {{
                            display: true,
                            text: '执行时间 (毫秒)'
                        }}
                    }}
                }},
                plugins: {{
                    legend: {{
                        display: false
                    }}
                }}
            }}
        }});
        
        // 层级性能对比图表
        const tierCtx = document.getElementById('tierChart').getContext('2d');
        new Chart(tierCtx, {{
            type: 'doughnut',
            data: {{
                labels: {tier_names},
                datasets: [{{
                    label: '基准测试数量',
                    data: {tier_data},
                    backgroundColor: [
                        'rgba(255, 99, 132, 0.6)',
                        'rgba(54, 162, 235, 0.6)', 
                        'rgba(255, 205, 86, 0.6)',
                        'rgba(75, 192, 192, 0.6)'
                    ],
                    borderColor: [
                        'rgba(255, 99, 132, 1)',
                        'rgba(54, 162, 235, 1)',
                        'rgba(255, 205, 86, 1)', 
                        'rgba(75, 192, 192, 1)'
                    ],
                    borderWidth: 1
                }}]
            }},
            options: {{
                responsive: true,
                plugins: {{
                    legend: {{
                        position: 'right'
                    }}
                }}
            }}
        }});
    </script>
</body>
</html>
"""

def format_duration(duration_ns: float) -> str:
    """格式化持续时间显示"""
    if duration_ns >= 1_000_000_000:  # >= 1s
        return f"{duration_ns / 1_000_000_000:.2f}s"
    elif duration_ns >= 1_000_000:  # >= 1ms  
        return f"{duration_ns / 1_000_000:.2f}ms"
    elif duration_ns >= 1_000:  # >= 1µs
        return f"{duration_ns / 1_000:.2f}µs" 
    else:
        return f"{duration_ns:.0f}ns"

def get_performance_badge(duration_ms: float) -> str:
    """根据执行时间返回性能等级标记"""
    if duration_ms < 1:
        return '<span class="performance-badge badge-excellent">优秀</span>'
    elif duration_ms < 10:
        return '<span class="performance-badge badge-good">良好</span>'
    else:
        return '<span class="performance-badge badge-warning">需优化</span>'

def analyze_results(results: List[Dict]) -> Dict:
    """分析基准测试结果"""
    if not results:
        return {}
    
    # 按crate分组
    crate_groups = {}
    tier_groups = {'foundation': [], 'core-logic': [], 'service': [], 'integration': []}
    
    for result in results:
        crate_name = result['crate_name']
        if crate_name not in crate_groups:
            crate_groups[crate_name] = []
        crate_groups[crate_name].append(result)
        
        # 分类到层级
        if any(x in crate_name for x in ['model', 'derive', 'macro']):
            tier_groups['foundation'].append(result)
        elif any(x in crate_name for x in ['transform', 'expression', 'template']):
            tier_groups['core-logic'].append(result)  
        elif any(x in crate_name for x in ['state', 'engine', 'file', 'search', 'persistence']):
            tier_groups['service'].append(result)
        elif any(x in crate_name for x in ['core', 'collaboration']):
            tier_groups['integration'].append(result)
    
    # 计算统计信息
    durations = [result['duration_ns'] for result in results]
    
    analysis = {
        'total_benchmarks': len(results),
        'total_crates': len(crate_groups),
        'total_tiers': sum(1 for tier_results in tier_groups.values() if tier_results),
        'avg_duration_ns': statistics.mean(durations),
        'median_duration_ns': statistics.median(durations),
        'min_duration_ns': min(durations),
        'max_duration_ns': max(durations),
        'crate_groups': crate_groups,
        'tier_groups': tier_groups
    }
    
    return analysis

def generate_tier_sections(tier_groups: Dict[str, List[Dict]]) -> str:
    """生成各层级的详细报告部分"""
    tier_names = {
        'foundation': '🏗️ 基础层 (Foundation)',
        'core-logic': '⚙️ 核心逻辑层 (Core Logic)', 
        'service': '🔧 服务层 (Service)',
        'integration': '🔗 集成层 (Integration)'
    }
    
    sections = []
    
    for tier_key, tier_results in tier_groups.items():
        if not tier_results:
            continue
            
        tier_name = tier_names.get(tier_key, tier_key)
        
        # 按crate分组
        crate_groups = {}
        for result in tier_results:
            crate_name = result['crate_name']
            if crate_name not in crate_groups:
                crate_groups[crate_name] = []
            crate_groups[crate_name].append(result)
        
        # 生成crate卡片
        crate_cards = []
        for crate_name, crate_results in sorted(crate_groups.items()):
            benchmark_items = []
            for result in sorted(crate_results, key=lambda x: x['benchmark_name']):
                duration_str = format_duration(result['duration_ns'])
                benchmark_items.append(f"""
                    <li class="benchmark-item">
                        <span class="benchmark-name">{result['benchmark_name']}</span>
                        <span class="benchmark-time">{duration_str}</span>
                    </li>
                """)
            
            avg_duration = statistics.mean([r['duration_ns'] for r in crate_results])
            badge = get_performance_badge(avg_duration / 1_000_000)  # 转换为毫秒
            
            crate_cards.append(f"""
                <div class="crate-card">
                    <div class="crate-name">{crate_name} {badge}</div>
                    <ul class="benchmark-list">
                        {''.join(benchmark_items)}
                    </ul>
                </div>
            """)
        
        sections.append(f"""
            <div class="tier-section">
                <div class="tier-title">{tier_name}</div>
                <div class="crate-grid">
                    {''.join(crate_cards)}
                </div>
            </div>
        """)
    
    return ''.join(sections)

def main():
    parser = argparse.ArgumentParser(description='生成综合性能报告')
    parser.add_argument('--input', required=True, help='输入JSON文件路径')
    parser.add_argument('--output', required=True, help='输出HTML文件路径')
    parser.add_argument('--title', default='ModuForge-RS 性能报告', help='报告标题')
    parser.add_argument('--commit', required=True, help='Git提交哈希')
    
    args = parser.parse_args()
    
    # 加载数据
    with open(args.input, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    # 提取结果列表
    if isinstance(data, list):
        results = data
    elif 'results' in data:
        results = data['results']
    else:
        print("错误: 无法识别的数据格式")
        return
    
    if not results:
        print("警告: 没有找到基准测试结果")
        return
    
    # 分析结果
    analysis = analyze_results(results)
    
    # 准备图表数据
    crate_durations = {}
    for crate_name, crate_results in analysis['crate_groups'].items():
        avg_duration = statistics.mean([r['duration_ns'] for r in crate_results])
        crate_durations[crate_name] = avg_duration / 1_000_000  # 转换为毫秒
    
    crate_names = list(crate_durations.keys())
    duration_data = list(crate_durations.values())
    
    # 层级数据
    tier_counts = {}
    for tier_name, tier_results in analysis['tier_groups'].items():
        if tier_results:
            tier_counts[tier_name] = len(tier_results)
    
    tier_names = list(tier_counts.keys())
    tier_data = list(tier_counts.values())
    
    # 生成层级部分
    tier_sections = generate_tier_sections(analysis['tier_groups'])
    
    # 生成HTML
    html_content = HTML_TEMPLATE.format(
        title=args.title,
        commit=args.commit[:8],  # 只显示前8位
        timestamp=datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S UTC'),
        total_benchmarks=analysis['total_benchmarks'],
        total_crates=analysis['total_crates'], 
        total_tiers=analysis['total_tiers'],
        avg_duration=format_duration(analysis['avg_duration_ns']),
        tier_sections=tier_sections,
        crate_names=json.dumps(crate_names),
        duration_data=json.dumps(duration_data),
        tier_names=json.dumps(tier_names),
        tier_data=json.dumps(tier_data)
    )
    
    # 保存HTML文件
    with open(args.output, 'w', encoding='utf-8') as f:
        f.write(html_content)
    
    print(f"✅ 综合报告生成完成:")
    print(f"   标题: {args.title}")
    print(f"   基准测试总数: {analysis['total_benchmarks']}")
    print(f"   测试的Crate数: {analysis['total_crates']}")
    print(f"   平均执行时间: {format_duration(analysis['avg_duration_ns'])}")
    print(f"   输出文件: {args.output}")

if __name__ == '__main__':
    main()