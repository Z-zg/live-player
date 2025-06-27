#!/bin/bash

# 开发环境启动脚本
# 同时启动服务器和客户端进行开发测试

set -e

echo "🚀 启动游戏直播开发环境..."
echo ""

# 检查 Rust 环境
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: 未找到 Cargo，请先安装 Rust"
    echo "安装方法: https://rustup.rs/"
    exit 1
fi

# 创建必要的目录
mkdir -p hls dash web

# 检查项目是否已编译
if [ ! -f "target/debug/game-stream-server" ] || [ ! -f "target/debug/game-stream-client" ]; then
    echo "🔨 编译项目..."
    cargo build
    echo "✅ 编译完成"
    echo ""
fi

# 启动服务器 (后台运行)
echo "🖥️  启动流媒体服务器..."
RUST_LOG=info cargo run --bin game-stream-server &
SERVER_PID=$!

# 等待服务器启动
sleep 3

# 检查服务器是否启动成功
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "❌ 服务器启动失败"
    exit 1
fi

echo "✅ 服务器启动成功 (PID: $SERVER_PID)"
echo "   RTMP: rtmp://localhost:1935/live"
echo "   HTTP: http://localhost:8080"
echo ""

# 启动客户端 (前台运行)
echo "📹 启动推流客户端..."
echo "按 Ctrl+C 停止推流"
echo ""

# 捕获中断信号，清理后台进程
trap 'echo ""; echo "🛑 停止服务..."; kill $SERVER_PID 2>/dev/null; exit 0' INT

RUST_LOG=info cargo run --bin game-stream-client

# 清理
kill $SERVER_PID 2>/dev/null
echo "✅ 开发环境已停止"
