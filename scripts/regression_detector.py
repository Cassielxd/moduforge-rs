#!/usr/bin/env python3
"""
简化的性能回归检测工具

用法:
python3 regression_detector.py current.json --baseline baseline.json --threshold 10.0
"""

import json
import argparse
import statistics
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass

@dataclass
class RegressionAlert:
    crate_name: str
    benchmark_name: str
    current_duration_ns: int
    baseline_duration_ns: int  
    change_percent: float
    severity: str
    
    def __str__(self):
        direction = "性能下降" if self.change_percent > 0 else "性能提升"
        return (f"{self.crate_name}/{self.benchmark_name}: "
                f"{direction} {abs(self.change_percent):.1f}% "
                f"({self.severity})")

class SimpleRegressionDetector:
    """简化的回归检测器"""
    
    def __init__(self, threshold_percent: float = 10.0):
        self.threshold_percent = threshold_percent
    
    def detect_regressions(
        self, 
        current_results: List[Dict], 
        baseline_results: List[Dict]
    ) -> List[RegressionAlert]:
        """检测性能回归"""
        
        # 构建基线数据查找表
        baseline_lookup = {}
        for result in baseline_results:
            key = f"{result['crate_name']}/{result['benchmark_name']}"
            baseline_lookup[key] = result['duration_ns']
        
        alerts = []
        
        for current in current_results:
            key = f"{current['crate_name']}/{current['benchmark_name']}"
            
            if key in baseline_lookup:
                current_duration = current['duration_ns']
                baseline_duration = baseline_lookup[key]
                
                # 计算变化百分比
                change_percent = ((current_duration - baseline_duration) / baseline_duration) * 100
                
                # 只报告性能下降（回归）
                if change_percent > self.threshold_percent:
                    severity = self._get_severity(change_percent)
                    
                    alert = RegressionAlert(
                        crate_name=current['crate_name'],
                        benchmark_name=current['benchmark_name'],
                        current_duration_ns=current_duration,
                        baseline_duration_ns=baseline_duration,
                        change_percent=change_percent,
                        severity=severity
                    )
                    
                    alerts.append(alert)
        
        return sorted(alerts, key=lambda x: x.change_percent, reverse=True)
    
    def _get_severity(self, change_percent: float) -> str:
        """根据变化百分比确定严重性"""
        if change_percent >= 50:
            return "CRITICAL"
        elif change_percent >= 25:
            return "HIGH"
        elif change_percent >= 15:
            return "MEDIUM"
        else:
            return "LOW"

def load_results(file_path: str) -> List[Dict]:
    """加载基准测试结果"""
    with open(file_path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    # 提取结果列表
    if isinstance(data, list):
        return data
    elif 'results' in data:
        return data['results']
    else:
        return []

def generate_regression_report(alerts: List[RegressionAlert], output_file: Optional[str] = None) -> str:
    """生成回归报告"""
    if not alerts:
        report = "✅ 未检测到性能回归"
    else:
        lines = [f"🚨 检测到 {len(alerts)} 个性能回归:"]
        
        # 按严重性分组
        severity_groups = {}
        for alert in alerts:
            if alert.severity not in severity_groups:
                severity_groups[alert.severity] = []
            severity_groups[alert.severity].append(alert)
        
        # 按严重性顺序输出
        for severity in ['CRITICAL', 'HIGH', 'MEDIUM', 'LOW']:
            if severity in severity_groups:
                lines.append(f"\n{severity} 级别:")
                for alert in severity_groups[severity]:
                    duration_change = format_duration_change(
                        alert.baseline_duration_ns, 
                        alert.current_duration_ns
                    )
                    lines.append(f"  - {alert.crate_name}/{alert.benchmark_name}")
                    lines.append(f"    变化: {alert.change_percent:+.1f}% ({duration_change})")
        
        report = '\n'.join(lines)
    
    if output_file:
        with open(output_file, 'w', encoding='utf-8') as f:
            f.write(report)
    
    return report

def format_duration_change(baseline_ns: int, current_ns: int) -> str:
    """格式化持续时间变化显示"""
    def format_ns(ns):
        if ns >= 1_000_000_000:
            return f"{ns / 1_000_000_000:.2f}s"
        elif ns >= 1_000_000:
            return f"{ns / 1_000_000:.2f}ms"
        elif ns >= 1_000:
            return f"{ns / 1_000:.2f}µs"
        else:
            return f"{ns}ns"
    
    return f"{format_ns(baseline_ns)} → {format_ns(current_ns)}"

def main():
    parser = argparse.ArgumentParser(description='性能回归检测工具')
    parser.add_argument('current', help='当前基准测试结果文件')
    parser.add_argument('--baseline', help='基线结果文件')
    parser.add_argument('--threshold', type=float, default=10.0, help='回归阈值百分比')
    parser.add_argument('--output', help='输出报告文件')
    parser.add_argument('--json', action='store_true', help='输出JSON格式')
    
    args = parser.parse_args()
    
    # 加载当前结果
    current_results = load_results(args.current)
    if not current_results:
        print("错误: 无法加载当前结果文件")
        return
    
    print(f"加载了 {len(current_results)} 个当前基准测试结果")
    
    if args.baseline:
        # 加载基线结果
        baseline_results = load_results(args.baseline)
        if not baseline_results:
            print("错误: 无法加载基线结果文件")
            return
            
        print(f"加载了 {len(baseline_results)} 个基线基准测试结果")
        
        # 执行回归检测
        detector = SimpleRegressionDetector(args.threshold)
        alerts = detector.detect_regressions(current_results, baseline_results)
        
        if args.json:
            # 输出JSON格式
            alerts_data = []
            for alert in alerts:
                alerts_data.append({
                    'crate_name': alert.crate_name,
                    'benchmark_name': alert.benchmark_name,
                    'current_duration_ns': alert.current_duration_ns,
                    'baseline_duration_ns': alert.baseline_duration_ns,
                    'change_percent': alert.change_percent,
                    'severity': alert.severity
                })
            
            result = {
                'threshold_percent': args.threshold,
                'total_regressions': len(alerts),
                'regressions': alerts_data
            }
            
            output_str = json.dumps(result, indent=2, ensure_ascii=False)
        else:
            # 生成文本报告
            output_str = generate_regression_report(alerts, args.output)
        
        if args.output:
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(output_str)
            print(f"报告已保存到: {args.output}")
        else:
            print(output_str)
            
        # 返回适当的退出码
        if alerts:
            exit(1)  # 有回归时返回非零退出码
        else:
            exit(0)
    else:
        # 只是分析当前结果
        print("\n📊 当前基准测试结果分析:")
        
        # 按crate分组
        crate_groups = {}
        for result in current_results:
            crate_name = result['crate_name']
            if crate_name not in crate_groups:
                crate_groups[crate_name] = []
            crate_groups[crate_name].append(result['duration_ns'])
        
        for crate_name in sorted(crate_groups.keys()):
            durations = crate_groups[crate_name]
            avg_duration = statistics.mean(durations)
            
            if avg_duration >= 1_000_000_000:
                duration_str = f"{avg_duration / 1_000_000_000:.2f}s"
            elif avg_duration >= 1_000_000:
                duration_str = f"{avg_duration / 1_000_000:.2f}ms"
            elif avg_duration >= 1_000:
                duration_str = f"{avg_duration / 1_000:.2f}µs"
            else:
                duration_str = f"{avg_duration:.0f}ns"
            
            print(f"  {crate_name}: {len(durations)} 个基准测试, 平均 {duration_str}")

if __name__ == '__main__':
    main()