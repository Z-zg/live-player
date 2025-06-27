#!/bin/bash

# 快速测试脚本 - 简单启动服务器进行测试

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

STREAM_KEY="test_stream_123"
SERVER_PID=""

# 清理函数
cleanup() {
    echo -e "\n${YELLOW}正在停止服务器...${NC}"
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
    fi
    pkill -f "game-stream-server" 2>/dev/null || true
    echo -e "${GREEN}服务器已停止${NC}"
}

trap cleanup EXIT INT TERM

echo -e "${BLUE}🎮 游戏直播系统 - 快速测试${NC}"
echo "================================"

# 检查可执行文件
if [ ! -f "./target/release/game-stream-server" ]; then
    echo -e "${YELLOW}正在编译项目...${NC}"
    cargo build --release
fi

# 启动服务器
echo -e "${CYAN}启动服务器...${NC}"
./target/release/game-stream-server -v &
SERVER_PID=$!

# 等待服务器启动
sleep 3

echo ""
echo -e "${GREEN}✅ 服务器启动成功！${NC}"
echo ""
echo -e "${CYAN}📱 测试方法:${NC}"
echo -e "1. 打开浏览器访问: ${YELLOW}http://localhost:8080${NC}"
echo -e "2. 输入流密钥: ${YELLOW}$STREAM_KEY${NC}"
echo -e "3. 选择播放协议并点击连接"
echo ""
echo -e "${CYAN}🎥 推流测试 (如果安装了 FFmpeg):${NC}"
echo -e "ffmpeg -f lavfi -i testsrc=size=1280x720:rate=30 \\"
echo -e "       -f lavfi -i sine=frequency=1000 \\"
echo -e "       -c:v libx264 -preset ultrafast \\"
echo -e "       -c:a aac -f flv \\"
echo -e "       rtmp://localhost:1935/live/$STREAM_KEY"
echo ""
echo -e "${CYAN}🔧 API 测试:${NC}"
echo -e "curl http://localhost:8080/api/streams"
echo ""
echo -e "${YELLOW}按 Ctrl+C 停止服务器${NC}"

# 保持运行
wait $SERVER_PID
