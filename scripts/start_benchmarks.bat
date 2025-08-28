@echo off
REM ModuForge-RS 基准测试启动脚本 (Windows版本)

setlocal EnableDelayedExpansion

REM 设置项目路径
set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%\..

REM 参数解析
set COMMAND=
set OUTPUT_DIR=benchmarks\results
set PARALLEL=1
set BASELINE_FILE=
set THRESHOLD=10.0
set CRATE_NAME=
set CURRENT_FILE=

:parse_args
if "%~1"=="" goto end_parse
if "%~1"=="-h" goto show_help
if "%~1"=="--help" goto show_help
if "%~1"=="-o" (
    set OUTPUT_DIR=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="--output" (
    set OUTPUT_DIR=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="-p" (
    set PARALLEL=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="--parallel" (
    set PARALLEL=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="--baseline" (
    set BASELINE_FILE=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="--threshold" (
    set THRESHOLD=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="all" (
    set COMMAND=benchmark
    set TIER=all
    shift
    goto parse_args
)
if "%~1"=="foundation" (
    set COMMAND=benchmark
    set TIER=foundation
    shift
    goto parse_args
)
if "%~1"=="core-logic" (
    set COMMAND=benchmark
    set TIER=core-logic
    shift
    goto parse_args
)
if "%~1"=="service" (
    set COMMAND=benchmark
    set TIER=service
    shift
    goto parse_args
)
if "%~1"=="integration" (
    set COMMAND=benchmark
    set TIER=integration
    shift
    goto parse_args
)
if "%~1"=="crate" (
    set COMMAND=crate
    set CRATE_NAME=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="report" (
    set COMMAND=report
    shift
    goto parse_args
)
if "%~1"=="detect" (
    set COMMAND=detect
    shift
    goto parse_args
)
if "%~1"=="setup" (
    set COMMAND=setup
    shift
    goto parse_args
)
REM 可能是detect命令的参数
if "!COMMAND!"=="detect" if "!CURRENT_FILE!"=="" (
    set CURRENT_FILE=%~1
    shift
    goto parse_args
)
shift
goto parse_args

:end_parse

REM 如果没有指定命令，显示帮助
if "!COMMAND!"=="" goto show_help

REM 检查依赖
echo [INFO] 检查依赖...

REM 检查Cargo
cargo --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Cargo未安装，请安装Rust工具链
    exit /b 1
)

REM 检查Python
python --version >nul 2>&1
if errorlevel 1 (
    echo [ERROR] Python未安装
    exit /b 1
)

echo [SUCCESS] 依赖检查通过

REM 切换到项目根目录
cd /d "%PROJECT_ROOT%"

REM 执行命令
if "!COMMAND!"=="setup" goto setup_cmd
if "!COMMAND!"=="benchmark" goto benchmark_cmd
if "!COMMAND!"=="crate" goto crate_cmd
if "!COMMAND!"=="report" goto report_cmd
if "!COMMAND!"=="detect" goto detect_cmd

echo [ERROR] 未知命令: !COMMAND!
goto show_help

:setup_cmd
echo [INFO] 设置基准测试环境...
mkdir "benchmarks\results" >nul 2>&1
mkdir "benchmarks\reports" >nul 2>&1
mkdir "benchmarks\baseline" >nul 2>&1
echo [SUCCESS] 环境设置完成
goto end

:benchmark_cmd
echo [INFO] 运行 !TIER! 基准测试...

REM 创建输出目录
mkdir "!OUTPUT_DIR!" >nul 2>&1

if "!TIER!"=="all" (
    echo [INFO] 运行全部基准测试...
    cargo run --bin benchmark-coordinator -- run-all --parallel !PARALLEL! --output-dir "!OUTPUT_DIR!"
) else if "!TIER!"=="foundation" (
    echo [INFO] 运行基础层基准测试...
    cargo bench --package moduforge-model
    cargo bench --package moduforge-macros-derive
    cargo bench --package moduforge-macros
) else if "!TIER!"=="core-logic" (
    echo [INFO] 运行核心逻辑层基准测试...
    cargo bench --package moduforge-transform
    cargo bench --package moduforge-rules-expression
    cargo bench --package moduforge-rules-template
) else if "!TIER!"=="service" (
    echo [INFO] 运行服务层基准测试...
    cargo bench --package moduforge-state
    cargo bench --package moduforge-rules-engine
    cargo bench --package moduforge-file
    cargo bench --package moduforge-search
    cargo bench --package moduforge-persistence
) else if "!TIER!"=="integration" (
    echo [INFO] 运行集成层基准测试...
    cargo bench --package moduforge-core
    cargo bench --package moduforge-collaboration
    cargo bench --package moduforge-collaboration-client
)

echo [SUCCESS] !TIER! 基准测试完成
goto end

:crate_cmd
if "!CRATE_NAME!"=="" (
    echo [ERROR] 请指定crate名称
    exit /b 1
)
echo [INFO] 运行 !CRATE_NAME! 基准测试...
cargo bench --package "!CRATE_NAME!"
echo [SUCCESS] !CRATE_NAME! 基准测试完成
goto end

:report_cmd
echo [INFO] 生成性能报告...

REM 查找最新的结果文件 (简化版本，Windows的dir命令限制)
set LATEST_RESULT=
for /f "delims=" %%i in ('dir /b /od "!OUTPUT_DIR!\*.json" 2^>nul') do set LATEST_RESULT=%%i

if "!LATEST_RESULT!"=="" (
    echo [WARNING] 未找到基准测试结果文件
    exit /b 1
)

set LATEST_RESULT=!OUTPUT_DIR!\!LATEST_RESULT!

REM 生成HTML报告
python "scripts\generate_comprehensive_report.py" --input "!LATEST_RESULT!" --output "!OUTPUT_DIR!\report.html" --title "ModuForge-RS 性能报告" --commit HEAD

echo [SUCCESS] 报告已生成: !OUTPUT_DIR!\report.html

REM 尝试在浏览器中打开
start "" "!OUTPUT_DIR!\report.html" >nul 2>&1

goto end

:detect_cmd
if "!CURRENT_FILE!"=="" (
    echo [ERROR] 请指定当前结果文件
    exit /b 1
)
if "!BASELINE_FILE!"=="" (
    echo [ERROR] 请指定基线文件 (--baseline)
    exit /b 1
)

echo [INFO] 检测性能回归...
echo [INFO] 当前结果: !CURRENT_FILE!
echo [INFO] 基线结果: !BASELINE_FILE!
echo [INFO] 阈值: !THRESHOLD!%%

mkdir "benchmarks\reports" >nul 2>&1

python "scripts\regression_detector.py" "!CURRENT_FILE!" --baseline "!BASELINE_FILE!" --threshold !THRESHOLD! --output "benchmarks\reports\regression_report.txt"

if errorlevel 1 (
    echo [WARNING] 检测到性能回归，请查看报告: benchmarks\reports\regression_report.txt
    exit /b 1
) else (
    echo [SUCCESS] 未检测到性能回归
)

goto end

:show_help
echo ModuForge-RS 基准测试工具
echo.
echo 用法:
echo   %0 [命令] [选项]
echo.
echo 命令:
echo   all              运行所有基准测试
echo   foundation       运行基础层基准测试
echo   core-logic       运行核心逻辑层基准测试
echo   service          运行服务层基准测试
echo   integration      运行集成层基准测试
echo   crate ^<name^>     运行指定crate的基准测试
echo   report           生成性能报告
echo   detect           检测性能回归
echo   setup            设置基准测试环境
echo.
echo 选项:
echo   -h, --help       显示此帮助信息
echo   -o, --output DIR 指定输出目录 (默认: benchmarks\results)
echo   -p, --parallel N 并行度 (默认: 1)
echo   --baseline FILE  基线文件路径
echo   --threshold N    回归检测阈值百分比 (默认: 10.0)
echo.
echo 示例:
echo   %0 all                           # 运行所有基准测试
echo   %0 foundation -o results         # 运行基础层测试，输出到results目录
echo   %0 crate moduforge-model         # 只测试model crate
echo   %0 detect current.json --baseline base.json  # 检测回归
echo.
goto end

:end
endlocal