#!/bin/bash
# ModuForge-RS 基准测试启动脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 显示带颜色的消息
log_info() {
    echo -e "${BLUE}ℹ️ $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️ $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 显示帮助信息
show_help() {
    echo "ModuForge-RS 基准测试工具"
    echo ""
    echo "用法:"
    echo "  $0 [命令] [选项]"
    echo ""
    echo "命令:"
    echo "  all              运行所有基准测试"
    echo "  foundation       运行基础层基准测试"
    echo "  core-logic       运行核心逻辑层基准测试"
    echo "  service          运行服务层基准测试"
    echo "  integration      运行集成层基准测试"
    echo "  crate <name>     运行指定crate的基准测试"
    echo "  report           生成性能报告"
    echo "  detect           检测性能回归"
    echo "  setup            设置基准测试环境"
    echo ""
    echo "选项:"
    echo "  -h, --help       显示此帮助信息"
    echo "  -o, --output DIR 指定输出目录 (默认: benchmarks/results)"
    echo "  -p, --parallel N 并行度 (默认: 1)"
    echo "  --baseline FILE  基线文件路径"
    echo "  --threshold N    回归检测阈值百分比 (默认: 10.0)"
    echo ""
    echo "示例:"
    echo "  $0 all                           # 运行所有基准测试"
    echo "  $0 foundation -o results         # 运行基础层测试，输出到results目录"
    echo "  $0 crate moduforge-model         # 只测试model crate"
    echo "  $0 detect --baseline base.json  # 检测回归"
    echo ""
}

# 检查依赖
check_dependencies() {
    log_info "检查依赖..."
    
    # 检查Rust工具链
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo未安装，请安装Rust工具链"
        exit 1
    fi
    
    # 检查Python
    if ! command -v python3 &> /dev/null; then
        log_error "Python3未安装"
        exit 1
    fi
    
    # 检查必要的Python包
    python3 -c "import pandas, matplotlib, scipy" 2>/dev/null || {
        log_warning "Python依赖包缺失，尝试安装..."
        pip3 install pandas matplotlib scipy numpy || {
            log_error "无法安装Python依赖包"
            exit 1
        }
    }
    
    log_success "依赖检查通过"
}

# 设置环境
setup_environment() {
    log_info "设置基准测试环境..."
    
    # 创建必要目录
    mkdir -p "$PROJECT_ROOT/benchmarks/results"
    mkdir -p "$PROJECT_ROOT/benchmarks/reports" 
    mkdir -p "$PROJECT_ROOT/benchmarks/baseline"
    
    # 初始化性能数据库
    python3 "$SCRIPT_DIR/performance_metrics.py" \
        --db "$PROJECT_ROOT/benchmarks/performance.db" \
        init 2>/dev/null || true
    
    log_success "环境设置完成"
}

# 运行基准测试
run_benchmark() {
    local tier="$1"
    local output_dir="${2:-benchmarks/results}"
    local parallel="${3:-1}"
    
    log_info "运行 $tier 基准测试..."
    
    cd "$PROJECT_ROOT"
    
    case "$tier" in
        "all")
            log_info "运行全部基准测试..."
            cargo run --bin benchmark-coordinator -- run-all \
                --parallel "$parallel" \
                --output-dir "$output_dir"
            ;;
        "foundation")
            log_info "运行基础层基准测试..."
            cargo bench --package moduforge-model
            cargo bench --package moduforge-macros-derive
            cargo bench --package moduforge-macros
            ;;
        "core-logic")
            log_info "运行核心逻辑层基准测试..."
            cargo bench --package moduforge-transform
            cargo bench --package moduforge-rules-expression
            cargo bench --package moduforge-rules-template
            ;;
        "service")
            log_info "运行服务层基准测试..."
            cargo bench --package moduforge-state
            cargo bench --package moduforge-rules-engine
            cargo bench --package moduforge-file
            cargo bench --package moduforge-search
            cargo bench --package moduforge-persistence
            ;;
        "integration")
            log_info "运行集成层基准测试..."
            cargo bench --package moduforge-core
            cargo bench --package moduforge-collaboration
            cargo bench --package moduforge-collaboration-client
            ;;
        *)
            log_error "未知的层级: $tier"
            exit 1
            ;;
    esac
    
    log_success "$tier 基准测试完成"
}

# 运行指定crate的基准测试
run_crate_benchmark() {
    local crate_name="$1"
    local output_dir="${2:-benchmarks/results}"
    
    log_info "运行 $crate_name 基准测试..."
    
    cd "$PROJECT_ROOT"
    cargo bench --package "$crate_name"
    
    log_success "$crate_name 基准测试完成"
}

# 生成报告
generate_report() {
    local output_dir="${1:-benchmarks/results}"
    
    log_info "生成性能报告..."
    
    cd "$PROJECT_ROOT"
    
    # 查找最新的结果文件
    latest_result=$(find "$output_dir" -name "*.json" -type f -printf '%T@ %p\n' | sort -n | tail -1 | cut -f2- -d" ")
    
    if [ -z "$latest_result" ]; then
        log_warning "未找到基准测试结果文件"
        return 1
    fi
    
    # 生成HTML报告
    python3 "$SCRIPT_DIR/generate_comprehensive_report.py" \
        --input "$latest_result" \
        --output "$output_dir/report.html" \
        --title "ModuForge-RS 性能报告" \
        --commit "$(git rev-parse HEAD)"
    
    log_success "报告已生成: $output_dir/report.html"
    
    # 尝试在浏览器中打开
    if command -v xdg-open &> /dev/null; then
        xdg-open "$output_dir/report.html"
    elif command -v open &> /dev/null; then
        open "$output_dir/report.html"
    fi
}

# 检测回归
detect_regression() {
    local current_file="$1"
    local baseline_file="$2"
    local threshold="${3:-10.0}"
    
    if [ -z "$current_file" ]; then
        log_error "请指定当前结果文件"
        exit 1
    fi
    
    if [ -z "$baseline_file" ]; then
        log_error "请指定基线文件 (--baseline)"
        exit 1
    fi
    
    log_info "检测性能回归..."
    log_info "当前结果: $current_file"
    log_info "基线结果: $baseline_file"
    log_info "阈值: $threshold%"
    
    cd "$PROJECT_ROOT"
    
    python3 "$SCRIPT_DIR/regression_detector.py" \
        "$current_file" \
        --baseline "$baseline_file" \
        --threshold "$threshold" \
        --output "benchmarks/reports/regression_report.txt"
    
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        log_success "未检测到性能回归"
    else
        log_warning "检测到性能回归，请查看报告: benchmarks/reports/regression_report.txt"
    fi
    
    return $exit_code
}

# 主函数
main() {
    local command=""
    local output_dir="benchmarks/results"
    local parallel=1
    local baseline_file=""
    local threshold="10.0"
    local crate_name=""
    
    # 解析命令行参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_help
                exit 0
                ;;
            -o|--output)
                output_dir="$2"
                shift 2
                ;;
            -p|--parallel)
                parallel="$2"
                shift 2
                ;;
            --baseline)
                baseline_file="$2"
                shift 2
                ;;
            --threshold)
                threshold="$2"
                shift 2
                ;;
            all|foundation|core-logic|service|integration)
                command="benchmark"
                tier="$1"
                shift
                ;;
            crate)
                command="crate"
                crate_name="$2"
                shift 2
                ;;
            report)
                command="report"
                shift
                ;;
            detect)
                command="detect"
                shift
                ;;
            setup)
                command="setup"
                shift
                ;;
            *)
                if [ -z "$command" ]; then
                    log_error "未知命令: $1"
                    show_help
                    exit 1
                else
                    # 可能是detect命令的参数
                    if [ "$command" = "detect" ] && [ -z "$current_file" ]; then
                        current_file="$1"
                    fi
                    shift
                fi
                ;;
        esac
    done
    
    # 如果没有指定命令，显示帮助
    if [ -z "$command" ]; then
        show_help
        exit 0
    fi
    
    # 检查依赖
    check_dependencies
    
    # 执行命令
    case "$command" in
        "setup")
            setup_environment
            ;;
        "benchmark")
            setup_environment
            run_benchmark "$tier" "$output_dir" "$parallel"
            ;;
        "crate")
            if [ -z "$crate_name" ]; then
                log_error "请指定crate名称"
                exit 1
            fi
            setup_environment
            run_crate_benchmark "$crate_name" "$output_dir"
            ;;
        "report")
            generate_report "$output_dir"
            ;;
        "detect")
            detect_regression "$current_file" "$baseline_file" "$threshold"
            ;;
        *)
            log_error "未知命令: $command"
            show_help
            exit 1
            ;;
    esac
}

# 如果脚本被直接执行
if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
    main "$@"
fi