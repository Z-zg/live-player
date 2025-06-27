#!/bin/bash

# FFmpeg 推流测试脚本

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

# 配置
RTMP_URL="rtmp://localhost:1935/live"
STREAM_KEY="test_stream_123"
DURATION=30

echo -e "${BLUE}🎥 FFmpeg 推流测试${NC}"
echo "======================"

# 检查 FFmpeg
if ! command -v ffmpeg &> /dev/null; then
    echo -e "${RED}❌ FFmpeg 未安装${NC}"
    echo "请安装 FFmpeg:"
    echo "  macOS: brew install ffmpeg"
    echo "  Ubuntu: sudo apt install ffmpeg"
    echo "  Windows: winget install FFmpeg"
    exit 1
fi

echo -e "${GREEN}✅ FFmpeg 已安装: $(ffmpeg -version | head -1)${NC}"

# 检查服务器
if ! curl -s http://localhost:8080 >/dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  服务器未运行，正在启动...${NC}"
    echo "请在另一个终端运行: ./scripts/quick-test.sh"
    echo "或者运行: ./target/release/game-stream-server"
    exit 1
fi

echo -e "${GREEN}✅ 服务器正在运行${NC}"

echo ""
echo -e "${CYAN}推流配置:${NC}"
echo -e "  RTMP URL: ${YELLOW}$RTMP_URL/$STREAM_KEY${NC}"
echo -e "  持续时间: ${YELLOW}${DURATION}秒${NC}"
echo -e "  视频: 1280x720@30fps, H.264"
echo -e "  音频: 44.1kHz, AAC"

echo ""
echo -e "${YELLOW}按 Enter 开始推流，或 Ctrl+C 取消...${NC}"
read -r

echo ""
echo -e "${CYAN}🚀 开始推流...${NC}"

# 方法1: 测试图案 + 正弦波音频
echo -e "${BLUE}方法1: 测试图案推流${NC}"
ffmpeg -f lavfi -i testsrc=duration=$DURATION:size=1280x720:rate=30 \
       -f lavfi -i sine=frequency=1000:duration=$DURATION \
       -c:v libx264 -preset ultrafast -tune zerolatency \
       -b:v 2000k -maxrate 2000k -bufsize 4000k \
       -c:a aac -b:a 128k -ar 44100 \
       -f flv "$RTMP_URL/$STREAM_KEY" 2>/dev/null || {
    echo -e "${RED}❌ 推流失败${NC}"
    exit 1
}

echo -e "${GREEN}✅ 推流完成！${NC}"

echo ""
echo -e "${CYAN}📺 观看方法:${NC}"
echo -e "1. 浏览器访问: ${YELLOW}http://localhost:8080${NC}"
echo -e "2. 输入流密钥: ${YELLOW}$STREAM_KEY${NC}"
echo -e "3. 选择播放协议并连接"

echo ""
echo -e "${CYAN}🔄 其他推流方法:${NC}"
echo ""

# 方法2: 屏幕录制 (macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo -e "${BLUE}方法2: 屏幕录制推流 (macOS)${NC}"
    echo "ffmpeg -f avfoundation -i \"1:0\" \\"
    echo "       -c:v libx264 -preset ultrafast \\"
    echo "       -c:a aac -f flv \\"
    echo "       $RTMP_URL/$STREAM_KEY"
    echo ""
fi

# 方法3: 摄像头推流
echo -e "${BLUE}方法3: 摄像头推流${NC}"
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "ffmpeg -f avfoundation -i \"0:0\" \\"
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "ffmpeg -f v4l2 -i /dev/video0 \\"
else
    echo "ffmpeg -f dshow -i video=\"摄像头名称\" \\"
fi
echo "       -c:v libx264 -preset ultrafast \\"
echo "       -c:a aac -f flv \\"
echo "       $RTMP_URL/$STREAM_KEY"

echo ""
echo -e "${BLUE}方法4: 文件推流${NC}"
echo "ffmpeg -re -i your_video.mp4 \\"
echo "       -c:v libx264 -c:a aac \\"
echo "       -f flv $RTMP_URL/$STREAM_KEY"

echo ""
echo -e "${GREEN}🎉 测试完成！${NC}"
