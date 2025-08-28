#!/usr/bin/env python3
"""
解析Criterion基准测试输出并转换为JSON格式

用法:
python3 parse_benchmark_output.py --input-dir benchmarks/results --output results.json --tier foundation --commit abc123
"""

import re
import json
import glob
import argparse
from datetime import datetime
from typing import List, Dict, Optional

class BenchmarkParser:
    """Criterion输出解析器"""
    
    def __init__(self):
        # Criterion输出格式正则表达式
        self.benchmark_pattern = re.compile(
            r'^([^\s]+)\s+time:\s+\[([0-9.]+)\s+([a-zA-Z]+)\s+([0-9.]+)\s+([a-zA-Z]+)\s+([0-9.]+)\s+([a-zA-Z]+)\]'
        )
        
        # 时间单位转换为纳秒
        self.time_units = {
            'ns': 1,
            'µs': 1000,
            'us': 1000,
            'ms': 1_000_000,
            's': 1_000_000_000
        }
    
    def parse_file(self, file_path: str, crate_name: str) -> List[Dict]:
        """解析单个基准测试输出文件"""
        results = []
        
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                
            for line in content.splitlines():
                match = self.benchmark_pattern.match(line.strip())
                if match:
                    benchmark_name = match.group(1)
                    min_time = float(match.group(2))
                    min_unit = match.group(3)
                    avg_time = float(match.group(4))
                    avg_unit = match.group(5)
                    max_time = float(match.group(6))
                    max_unit = match.group(7)
                    
                    # 转换为纳秒
                    min_ns = int(min_time * self.time_units.get(min_unit, 1))
                    avg_ns = int(avg_time * self.time_units.get(avg_unit, 1))
                    max_ns = int(max_time * self.time_units.get(max_unit, 1))
                    
                    result = {
                        'crate_name': crate_name,
                        'benchmark_name': benchmark_name,
                        'duration_ns': avg_ns,
                        'min_duration_ns': min_ns,
                        'max_duration_ns': max_ns,
                        'memory_usage_bytes': 0,  # Criterion不直接提供内存信息
                        'cpu_utilization_percent': 0.0,
                        'timestamp': datetime.utcnow().isoformat() + 'Z',
                        'metadata': {
                            'min_time': f"{min_time} {min_unit}",
                            'avg_time': f"{avg_time} {avg_unit}",
                            'max_time': f"{max_time} {max_unit}",
                            'source_file': file_path
                        }
                    }
                    
                    results.append(result)
                    
        except Exception as e:
            print(f"警告: 解析文件 {file_path} 时出错: {e}")
        
        return results
    
    def extract_crate_name_from_filename(self, filename: str) -> str:
        """从文件名提取crate名称"""
        # 移除路径和扩展名
        basename = filename.split('/')[-1].replace('-output.txt', '')
        
        # 映射常见的文件名到crate名称
        name_mapping = {
            'model': 'moduforge-model',
            'derive': 'moduforge-macros-derive', 
            'macro': 'moduforge-macros',
            'transform': 'moduforge-transform',
            'expression': 'moduforge-rules-expression',
            'template': 'moduforge-rules-template',
            'state': 'moduforge-state',
            'engine': 'moduforge-rules-engine',
            'file': 'moduforge-file',
            'search': 'moduforge-search',
            'persistence': 'moduforge-persistence',
            'core': 'moduforge-core',
            'collaboration': 'moduforge-collaboration',
            'collaboration-client': 'moduforge-collaboration-client'
        }
        
        return name_mapping.get(basename, basename)

def get_tier_crates(tier: str) -> List[str]:
    """根据层级返回对应的crate列表"""
    tier_mapping = {
        'foundation': [
            'moduforge-model',
            'moduforge-macros-derive', 
            'moduforge-macros'
        ],
        'core-logic': [
            'moduforge-transform',
            'moduforge-rules-expression',
            'moduforge-rules-template'
        ],
        'service': [
            'moduforge-state',
            'moduforge-rules-engine',
            'moduforge-file',
            'moduforge-search',
            'moduforge-persistence'
        ],
        'integration': [
            'moduforge-core',
            'moduforge-collaboration',
            'moduforge-collaboration-client'
        ]
    }
    
    return tier_mapping.get(tier, [])

def main():
    parser = argparse.ArgumentParser(description='解析Criterion基准测试输出')
    parser.add_argument('--input-dir', required=True, help='输入目录路径')
    parser.add_argument('--output', required=True, help='输出JSON文件路径') 
    parser.add_argument('--tier', required=True, help='执行层级')
    parser.add_argument('--commit', required=True, help='Git提交哈希')
    
    args = parser.parse_args()
    
    # 创建解析器
    parser_instance = BenchmarkParser()
    all_results = []
    
    # 获取当前层级的crate列表
    tier_crates = get_tier_crates(args.tier)
    
    # 查找并解析所有输出文件
    output_files = glob.glob(f"{args.input_dir}/*-output.txt")
    
    for file_path in output_files:
        crate_name = parser_instance.extract_crate_name_from_filename(file_path)
        
        # 只处理当前层级的crate
        if crate_name in tier_crates:
            print(f"解析 {crate_name} 的基准测试输出: {file_path}")
            results = parser_instance.parse_file(file_path, crate_name)
            
            # 添加提交信息
            for result in results:
                result['git_commit'] = args.commit
            
            all_results.extend(results)
            print(f"  找到 {len(results)} 个基准测试结果")
    
    # 生成汇总信息
    summary = {
        'tier': args.tier,
        'timestamp': datetime.utcnow().isoformat() + 'Z',
        'git_commit': args.commit,
        'total_benchmarks': len(all_results),
        'crates_processed': len(set(r['crate_name'] for r in all_results)),
        'results': all_results
    }
    
    # 保存结果
    with open(args.output, 'w', encoding='utf-8') as f:
        json.dump(summary, f, indent=2, ensure_ascii=False)
    
    print(f"✅ 解析完成:")
    print(f"   层级: {args.tier}")
    print(f"   处理的crate数量: {summary['crates_processed']}")
    print(f"   基准测试总数: {summary['total_benchmarks']}")
    print(f"   输出文件: {args.output}")
    
    # 显示各crate的基准测试数量
    crate_counts = {}
    for result in all_results:
        crate_name = result['crate_name']
        crate_counts[crate_name] = crate_counts.get(crate_name, 0) + 1
    
    print(f"   详细统计:")
    for crate_name, count in sorted(crate_counts.items()):
        print(f"     {crate_name}: {count} 个基准测试")

if __name__ == '__main__':
    main()