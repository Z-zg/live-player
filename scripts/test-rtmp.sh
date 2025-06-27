#!/bin/bash

# RTMP 推流测试脚本
# 使用 FFmpeg 生成测试视频流并推送到服务器

set -e

# 配置参数
RTMP_URL="rtmp://localhost:1935/live"
STREAM_KEY="test_stream"
DURATION=30  # 测试时长(秒)

echo "🎬 开始 RTMP 推流测试..."
echo "服务器地址: $RTMP_URL"
echo "流密钥: $STREAM_KEY"
echo "测试时长: ${DURATION}秒"
echo ""

# 检查 FFmpeg 是否安装
if ! command -v ffmpeg &> /dev/null; then
    echo "❌ 错误: 未找到 FFmpeg，请先安装 FFmpeg"
    echo ""
    echo "安装方法:"
    echo "  macOS: brew install ffmpeg"
    echo "  Ubuntu: sudo apt install ffmpeg"
    echo "  Windows: winget install FFmpeg"
    exit 1
fi

# 检查服务器是否运行
echo "🔍 检查服务器连接..."
if ! nc -z localhost 1935 2>/dev/null; then
    echo "❌ 错误: 无法连接到 RTMP 服务器 (localhost:1935)"
    echo "请确保游戏直播服务器正在运行:"
    echo "  cargo run --bin game-stream-server"
    exit 1
fi

echo "✅ 服务器连接正常"
echo ""

# 生成测试视频流
echo "🚀 开始推流测试..."
echo "按 Ctrl+C 停止推流"
echo ""

ffmpeg \
    -f lavfi -i "testsrc2=duration=${DURATION}:size=1280x720:rate=30" \
    -f lavfi -i "sine=frequency=1000:duration=${DURATION}" \
    -c:v libx264 -preset fast -tune zerolatency \
    -pix_fmt yuv420p -g 60 -keyint_min 60 \
    -b:v 1000k -maxrate 1000k -bufsize 2000k \
    -c:a aac -b:a 128k -ar 44100 -ac 2 \
    -f flv "${RTMP_URL}/${STREAM_KEY}" \
    -y

echo ""
echo "✅ RTMP 推流测试完成"
echo ""
echo "📺 现在可以通过以下方式观看:"
echo "  Web 观看端: http://localhost:8080"
echo "  流密钥: $STREAM_KEY"
