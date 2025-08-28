#!/usr/bin/env python3
"""
ModuForge-RS 性能指标收集和分析系统

功能：
- 收集基准测试结果
- 存储历史数据
- 性能回归检测
- 生成可视化报告
"""

import sqlite3
import json
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Dict, Optional, Tuple
import argparse
import logging
from dataclasses import dataclass
from scipy import stats

# 配置日志
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class BenchmarkResult:
    """基准测试结果数据类"""
    crate_name: str
    benchmark_name: str
    duration_ns: int
    memory_usage_bytes: int
    cpu_utilization_percent: float
    timestamp: str
    git_commit: Optional[str] = None
    metadata: Optional[Dict] = None

class PerformanceDatabase:
    """性能数据库管理器"""
    
    def __init__(self, db_path: str):
        self.db_path = db_path
        self.init_database()
    
    def init_database(self):
        """初始化数据库表结构"""
        conn = sqlite3.connect(self.db_path)
        
        # 基准测试结果表
        conn.execute('''
            CREATE TABLE IF NOT EXISTS benchmark_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                crate_name TEXT NOT NULL,
                benchmark_name TEXT NOT NULL,
                duration_ns INTEGER NOT NULL,
                memory_usage_bytes INTEGER DEFAULT 0,
                cpu_utilization_percent REAL DEFAULT 0.0,
                timestamp DATETIME NOT NULL,
                git_commit TEXT,
                metadata_json TEXT,
                UNIQUE(crate_name, benchmark_name, timestamp)
            )
        ''')
        
        # 性能基线表
        conn.execute('''
            CREATE TABLE IF NOT EXISTS performance_baselines (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                crate_name TEXT NOT NULL,
                benchmark_name TEXT NOT NULL,
                baseline_duration_ns INTEGER NOT NULL,
                baseline_timestamp DATETIME NOT NULL,
                git_commit TEXT,
                is_active BOOLEAN DEFAULT 1,
                UNIQUE(crate_name, benchmark_name, is_active)
            )
        ''')
        
        # 回归警报表
        conn.execute('''
            CREATE TABLE IF NOT EXISTS regression_alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                crate_name TEXT NOT NULL,
                benchmark_name TEXT NOT NULL,
                current_duration_ns INTEGER NOT NULL,
                baseline_duration_ns INTEGER NOT NULL,
                regression_percent REAL NOT NULL,
                severity TEXT NOT NULL,
                timestamp DATETIME NOT NULL,
                git_commit TEXT,
                resolved BOOLEAN DEFAULT 0
            )
        ''')
        
        # 创建索引
        conn.execute('CREATE INDEX IF NOT EXISTS idx_results_crate_time ON benchmark_results(crate_name, timestamp)')
        conn.execute('CREATE INDEX IF NOT EXISTS idx_results_benchmark ON benchmark_results(benchmark_name)')
        conn.execute('CREATE INDEX IF NOT EXISTS idx_alerts_unresolved ON regression_alerts(resolved, timestamp)')
        
        conn.commit()
        conn.close()
        
        logger.info(f"数据库初始化完成: {self.db_path}")
    
    def insert_results(self, results: List[BenchmarkResult]):
        """插入基准测试结果"""
        conn = sqlite3.connect(self.db_path)
        
        for result in results:
            metadata_json = json.dumps(result.metadata) if result.metadata else None
            
            conn.execute('''
                INSERT OR REPLACE INTO benchmark_results
                (crate_name, benchmark_name, duration_ns, memory_usage_bytes, 
                 cpu_utilization_percent, timestamp, git_commit, metadata_json)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                result.crate_name,
                result.benchmark_name, 
                result.duration_ns,
                result.memory_usage_bytes,
                result.cpu_utilization_percent,
                result.timestamp,
                result.git_commit,
                metadata_json
            ))
        
        conn.commit()
        conn.close()
        
        logger.info(f"插入 {len(results)} 条基准测试结果")
    
    def get_baseline(self, crate_name: str, benchmark_name: str) -> Optional[Tuple[int, str]]:
        """获取性能基线"""
        conn = sqlite3.connect(self.db_path)
        
        result = conn.execute('''
            SELECT baseline_duration_ns, git_commit
            FROM performance_baselines
            WHERE crate_name = ? AND benchmark_name = ? AND is_active = 1
        ''', (crate_name, benchmark_name)).fetchone()
        
        conn.close()
        
        return result if result else None
    
    def set_baseline(self, crate_name: str, benchmark_name: str, duration_ns: int, git_commit: str):
        """设置性能基线"""
        conn = sqlite3.connect(self.db_path)
        
        # 禁用现有基线
        conn.execute('''
            UPDATE performance_baselines 
            SET is_active = 0 
            WHERE crate_name = ? AND benchmark_name = ?
        ''', (crate_name, benchmark_name))
        
        # 插入新基线
        conn.execute('''
            INSERT INTO performance_baselines
            (crate_name, benchmark_name, baseline_duration_ns, baseline_timestamp, git_commit)
            VALUES (?, ?, ?, ?, ?)
        ''', (crate_name, benchmark_name, duration_ns, datetime.now().isoformat(), git_commit))
        
        conn.commit()
        conn.close()
        
        logger.info(f"设置基线: {crate_name}/{benchmark_name} = {duration_ns}ns")

class RegressionDetector:
    """性能回归检测器"""
    
    def __init__(self, db: PerformanceDatabase, threshold_percent: float = 10.0):
        self.db = db
        self.threshold_percent = threshold_percent
    
    def detect_regressions(self, results: List[BenchmarkResult]) -> List[Dict]:
        """检测性能回归"""
        alerts = []
        
        for result in results:
            baseline = self.db.get_baseline(result.crate_name, result.benchmark_name)
            
            if baseline:
                baseline_duration, baseline_commit = baseline
                current_duration = result.duration_ns
                
                # 计算变化百分比
                change_percent = ((current_duration - baseline_duration) / baseline_duration) * 100
                
                if change_percent > self.threshold_percent:
                    severity = self._get_severity(change_percent)
                    
                    alert = {
                        'crate_name': result.crate_name,
                        'benchmark_name': result.benchmark_name,
                        'current_duration_ns': current_duration,
                        'baseline_duration_ns': baseline_duration,
                        'regression_percent': change_percent,
                        'severity': severity,
                        'timestamp': result.timestamp,
                        'git_commit': result.git_commit,
                        'baseline_commit': baseline_commit
                    }
                    
                    alerts.append(alert)
                    self._save_alert(alert)
        
        return alerts
    
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
    
    def _save_alert(self, alert: Dict):
        """保存回归警报"""
        conn = sqlite3.connect(self.db.db_path)
        
        conn.execute('''
            INSERT INTO regression_alerts
            (crate_name, benchmark_name, current_duration_ns, baseline_duration_ns,
             regression_percent, severity, timestamp, git_commit)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ''', (
            alert['crate_name'],
            alert['benchmark_name'],
            alert['current_duration_ns'],
            alert['baseline_duration_ns'],
            alert['regression_percent'],
            alert['severity'],
            alert['timestamp'],
            alert['git_commit']
        ))
        
        conn.commit()
        conn.close()

class PerformanceAnalyzer:
    """性能分析器"""
    
    def __init__(self, db: PerformanceDatabase):
        self.db = db
    
    def generate_trend_report(self, crate_name: str, days: int = 30) -> Dict:
        """生成性能趋势报告"""
        conn = sqlite3.connect(self.db.db_path)
        
        # 获取指定时间范围内的数据
        end_date = datetime.now()
        start_date = end_date - timedelta(days=days)
        
        df = pd.read_sql_query('''
            SELECT benchmark_name, duration_ns, timestamp
            FROM benchmark_results
            WHERE crate_name = ? AND timestamp BETWEEN ? AND ?
            ORDER BY timestamp
        ''', conn, params=(crate_name, start_date.isoformat(), end_date.isoformat()))
        
        conn.close()
        
        if df.empty:
            return {'error': f'没有找到 {crate_name} 的数据'}
        
        # 计算趋势统计
        report = {
            'crate_name': crate_name,
            'period_days': days,
            'total_benchmarks': len(df['benchmark_name'].unique()),
            'total_runs': len(df),
            'benchmarks': {}
        }
        
        for benchmark in df['benchmark_name'].unique():
            benchmark_data = df[df['benchmark_name'] == benchmark]
            durations = benchmark_data['duration_ns'].values
            
            # 统计信息
            stats_info = {
                'count': len(durations),
                'mean_ns': float(np.mean(durations)),
                'median_ns': float(np.median(durations)),
                'std_ns': float(np.std(durations)),
                'min_ns': int(np.min(durations)),
                'max_ns': int(np.max(durations)),
            }
            
            # 趋势分析
            if len(durations) >= 2:
                # 线性回归分析趋势
                x = np.arange(len(durations))
                slope, intercept, r_value, p_value, std_err = stats.linregress(x, durations)
                
                trend_info = {
                    'trend_slope': slope,
                    'trend_r_squared': r_value ** 2,
                    'trend_p_value': p_value,
                    'is_improving': slope < 0,  # 时间增长但性能提升（duration减少）
                    'is_degrading': slope > 0 and p_value < 0.05  # 统计显著的性能下降
                }
                
                stats_info.update(trend_info)
            
            report['benchmarks'][benchmark] = stats_info
        
        return report
    
    def generate_comparison_report(self, base_commit: str, current_commit: str) -> Dict:
        """生成版本对比报告"""
        conn = sqlite3.connect(self.db.db_path)
        
        # 获取两个版本的数据
        base_data = pd.read_sql_query('''
            SELECT crate_name, benchmark_name, AVG(duration_ns) as avg_duration
            FROM benchmark_results
            WHERE git_commit = ?
            GROUP BY crate_name, benchmark_name
        ''', conn, params=(base_commit,))
        
        current_data = pd.read_sql_query('''
            SELECT crate_name, benchmark_name, AVG(duration_ns) as avg_duration
            FROM benchmark_results
            WHERE git_commit = ?
            GROUP BY crate_name, benchmark_name
        ''', conn, params=(current_commit,))
        
        conn.close()
        
        # 合并数据进行比较
        comparison = pd.merge(
            base_data,
            current_data,
            on=['crate_name', 'benchmark_name'],
            suffixes=('_base', '_current')
        )
        
        # 计算变化
        comparison['change_ns'] = comparison['avg_duration_current'] - comparison['avg_duration_base']
        comparison['change_percent'] = (comparison['change_ns'] / comparison['avg_duration_base']) * 100
        
        # 生成报告
        report = {
            'base_commit': base_commit,
            'current_commit': current_commit,
            'total_comparisons': len(comparison),
            'improvements': len(comparison[comparison['change_percent'] < -5]),  # 性能提升超过5%
            'regressions': len(comparison[comparison['change_percent'] > 5]),    # 性能下降超过5%
            'stable': len(comparison[comparison['change_percent'].abs() <= 5]),   # 变化在5%以内
            'details': comparison.to_dict('records')
        }
        
        return report
    
    def export_visualization(self, report: Dict, output_path: str):
        """导出可视化图表"""
        if 'benchmarks' not in report:
            logger.error("报告格式错误，无法生成图表")
            return
        
        crate_name = report['crate_name']
        benchmarks = report['benchmarks']
        
        # 创建子图
        fig, axes = plt.subplots(2, 2, figsize=(15, 12))
        fig.suptitle(f'{crate_name} 性能分析报告', fontsize=16, fontweight='bold')
        
        # 1. 平均执行时间条形图
        benchmark_names = list(benchmarks.keys())
        mean_times = [benchmarks[name]['mean_ns'] / 1_000_000 for name in benchmark_names]  # 转换为毫秒
        
        axes[0, 0].bar(range(len(benchmark_names)), mean_times, color='skyblue')
        axes[0, 0].set_title('平均执行时间')
        axes[0, 0].set_ylabel('时间 (毫秒)')
        axes[0, 0].set_xticks(range(len(benchmark_names)))
        axes[0, 0].set_xticklabels(benchmark_names, rotation=45, ha='right')
        
        # 2. 变异系数（稳定性指标）
        cv_values = []
        for name in benchmark_names:
            mean_val = benchmarks[name]['mean_ns']
            std_val = benchmarks[name]['std_ns']
            cv = (std_val / mean_val) * 100 if mean_val > 0 else 0
            cv_values.append(cv)
        
        axes[0, 1].bar(range(len(benchmark_names)), cv_values, color='lightcoral')
        axes[0, 1].set_title('变异系数 (稳定性)')
        axes[0, 1].set_ylabel('变异系数 (%)')
        axes[0, 1].set_xticks(range(len(benchmark_names)))
        axes[0, 1].set_xticklabels(benchmark_names, rotation=45, ha='right')
        
        # 3. 趋势分析
        trend_slopes = []
        for name in benchmark_names:
            if 'trend_slope' in benchmarks[name]:
                trend_slopes.append(benchmarks[name]['trend_slope'])
            else:
                trend_slopes.append(0)
        
        colors = ['green' if slope < 0 else 'red' for slope in trend_slopes]
        axes[1, 0].bar(range(len(benchmark_names)), trend_slopes, color=colors)
        axes[1, 0].set_title('性能趋势 (负值表示改善)')
        axes[1, 0].set_ylabel('趋势斜率')
        axes[1, 0].set_xticks(range(len(benchmark_names)))
        axes[1, 0].set_xticklabels(benchmark_names, rotation=45, ha='right')
        axes[1, 0].axhline(y=0, color='black', linestyle='--', alpha=0.5)
        
        # 4. 性能范围（最小值-最大值）
        min_times = [benchmarks[name]['min_ns'] / 1_000_000 for name in benchmark_names]
        max_times = [benchmarks[name]['max_ns'] / 1_000_000 for name in benchmark_names]
        
        axes[1, 1].fill_between(range(len(benchmark_names)), min_times, max_times, alpha=0.3, color='purple')
        axes[1, 1].plot(range(len(benchmark_names)), mean_times, color='purple', marker='o')
        axes[1, 1].set_title('性能范围 (最小值-最大值)')
        axes[1, 1].set_ylabel('时间 (毫秒)')
        axes[1, 1].set_xticks(range(len(benchmark_names)))
        axes[1, 1].set_xticklabels(benchmark_names, rotation=45, ha='right')
        
        plt.tight_layout()
        plt.savefig(output_path, dpi=300, bbox_inches='tight')
        plt.close()
        
        logger.info(f"可视化图表已保存到: {output_path}")

def load_benchmark_results(file_path: str) -> List[BenchmarkResult]:
    """从JSON文件加载基准测试结果"""
    with open(file_path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    results = []
    if isinstance(data, list):
        for item in data:
            result = BenchmarkResult(
                crate_name=item['crate_name'],
                benchmark_name=item['benchmark_name'],
                duration_ns=item['duration_ns'],
                memory_usage_bytes=item.get('memory_usage_bytes', 0),
                cpu_utilization_percent=item.get('cpu_utilization_percent', 0.0),
                timestamp=item['timestamp'],
                git_commit=item.get('git_commit'),
                metadata=item.get('metadata')
            )
            results.append(result)
    
    return results

def main():
    parser = argparse.ArgumentParser(description='ModuForge-RS 性能指标管理工具')
    parser.add_argument('--db', default='benchmarks/performance.db', help='数据库路径')
    
    subparsers = parser.add_subparsers(dest='command', help='可用命令')
    
    # 导入命令
    import_parser = subparsers.add_parser('import', help='导入基准测试结果')
    import_parser.add_argument('file', help='结果文件路径 (JSON格式)')
    
    # 基线命令
    baseline_parser = subparsers.add_parser('baseline', help='设置性能基线')
    baseline_parser.add_argument('--crate', required=True, help='Crate名称')
    baseline_parser.add_argument('--benchmark', required=True, help='基准测试名称')
    baseline_parser.add_argument('--duration', type=int, required=True, help='基线时间 (纳秒)')
    baseline_parser.add_argument('--commit', required=True, help='Git提交哈希')
    
    # 检测命令
    detect_parser = subparsers.add_parser('detect', help='检测性能回归')
    detect_parser.add_argument('file', help='当前结果文件路径')
    detect_parser.add_argument('--threshold', type=float, default=10.0, help='回归阈值百分比')
    
    # 报告命令
    report_parser = subparsers.add_parser('report', help='生成性能报告')
    report_parser.add_argument('--crate', required=True, help='Crate名称')
    report_parser.add_argument('--days', type=int, default=30, help='分析天数')
    report_parser.add_argument('--output', help='输出文件路径')
    report_parser.add_argument('--format', choices=['json', 'chart'], default='json', help='输出格式')
    
    # 比较命令
    compare_parser = subparsers.add_parser('compare', help='比较两个版本的性能')
    compare_parser.add_argument('--base', required=True, help='基准版本提交哈希')
    compare_parser.add_argument('--current', required=True, help='当前版本提交哈希')
    compare_parser.add_argument('--output', help='输出文件路径')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return
    
    # 初始化数据库
    db = PerformanceDatabase(args.db)
    
    if args.command == 'import':
        results = load_benchmark_results(args.file)
        db.insert_results(results)
        print(f"✅ 成功导入 {len(results)} 条结果")
    
    elif args.command == 'baseline':
        db.set_baseline(args.crate, args.benchmark, args.duration, args.commit)
        print(f"✅ 基线设置完成: {args.crate}/{args.benchmark}")
    
    elif args.command == 'detect':
        results = load_benchmark_results(args.file)
        detector = RegressionDetector(db, args.threshold)
        alerts = detector.detect_regressions(results)
        
        if alerts:
            print(f"🚨 检测到 {len(alerts)} 个性能回归:")
            for alert in alerts:
                print(f"  - {alert['crate_name']}/{alert['benchmark_name']}: "
                      f"{alert['regression_percent']:+.1f}% ({alert['severity']})")
        else:
            print("✅ 未检测到性能回归")
    
    elif args.command == 'report':
        analyzer = PerformanceAnalyzer(db)
        report = analyzer.generate_trend_report(args.crate, args.days)
        
        if args.format == 'json':
            output_file = args.output or f'{args.crate}_report.json'
            with open(output_file, 'w', encoding='utf-8') as f:
                json.dump(report, f, indent=2, ensure_ascii=False)
            print(f"📊 报告已保存到: {output_file}")
        elif args.format == 'chart':
            output_file = args.output or f'{args.crate}_chart.png'
            analyzer.export_visualization(report, output_file)
    
    elif args.command == 'compare':
        analyzer = PerformanceAnalyzer(db)
        report = analyzer.generate_comparison_report(args.base, args.current)
        
        output_file = args.output or f'comparison_{args.base[:8]}_vs_{args.current[:8]}.json'
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(report, f, indent=2, ensure_ascii=False)
        
        print(f"📊 对比报告已保存到: {output_file}")
        print(f"   改善: {report['improvements']}")
        print(f"   回归: {report['regressions']}")
        print(f"   稳定: {report['stable']}")

if __name__ == '__main__':
    main()