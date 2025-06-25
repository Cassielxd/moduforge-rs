@echo off
echo 启动 ModuForge Yjs 测试环境...

echo.
echo 1. 启动 Rust 后端服务器...
start "Rust Backend" cmd /k "cd /d ..\crates\transmission && cargo run --bin main"

echo.
echo 2. 等待 3 秒让后端启动...
timeout /t 3 /nobreak > nul

echo.
echo 3. 启动前端开发服务器...
start "Frontend Dev Server" cmd /k "npm run dev"

echo.
echo 4. 等待 3 秒让前端启动...
timeout /t 3 /nobreak > nul

echo.
echo 5. 打开浏览器...
start http://localhost:3000

echo.
echo 测试环境已启动！
echo - 前端: http://localhost:3000
echo - 后端: ws://localhost:8080
echo.
echo 按任意键退出...
pause > nul 