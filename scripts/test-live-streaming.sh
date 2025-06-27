#!/bin/bash

# 完整的直播测试脚本
# 此脚本将启动服务器、模拟推流、并提供测试指导

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 配置
SERVER_PORT=8080
RTMP_PORT=1935
STREAM_KEY="test_stream_123"
SERVER_PID=""
CLIENT_PID=""

# 日志函数
log() {
    echo -e "${CYAN}[$(date '+%H:%M:%S')]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[$(date '+%H:%M:%S')] ✅ $1${NC}"
}

log_error() {
    echo -e "${RED}[$(date '+%H:%M:%S')] ❌ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}[$(date '+%H:%M:%S')] ⚠️  $1${NC}"
}

log_info() {
    echo -e "${BLUE}[$(date '+%H:%M:%S')] ℹ️  $1${NC}"
}

# 清理函数
cleanup() {
    log "正在清理进程..."
    
    if [ ! -z "$SERVER_PID" ]; then
        kill $SERVER_PID 2>/dev/null || true
        log "服务器进程已停止"
    fi
    
    if [ ! -z "$CLIENT_PID" ]; then
        kill $CLIENT_PID 2>/dev/null || true
        log "客户端进程已停止"
    fi
    
    # 清理可能占用端口的进程
    pkill -f "game-stream-server" 2>/dev/null || true
    pkill -f "game-stream-client" 2>/dev/null || true
    
    log "清理完成"
}

# 设置信号处理
trap cleanup EXIT INT TERM

# 检查依赖
check_dependencies() {
    log "检查依赖..."
    
    # 检查可执行文件
    if [ ! -f "./target/release/game-stream-server" ]; then
        log_error "服务器可执行文件不存在，请先运行: cargo build --release"
        exit 1
    fi
    
    if [ ! -f "./target/release/game-stream-client" ]; then
        log_error "客户端可执行文件不存在，请先运行: cargo build --release"
        exit 1
    fi
    
    # 检查端口占用
    if lsof -i :$SERVER_PORT >/dev/null 2>&1; then
        log_error "端口 $SERVER_PORT 已被占用"
        exit 1
    fi
    
    if lsof -i :$RTMP_PORT >/dev/null 2>&1; then
        log_error "端口 $RTMP_PORT 已被占用"
        exit 1
    fi
    
    log_success "依赖检查通过"
}

# 启动服务器
start_server() {
    log "启动流媒体服务器..."
    
    # 创建临时配置文件
    cat > /tmp/test_server.toml << EOF
[server]
host = "0.0.0.0"
rtmp_port = $RTMP_PORT
http_port = $SERVER_PORT

[auth]
enabled = false

[hls]
enabled = true
segment_duration = 6
playlist_length = 5

[webrtc]
enabled = true
EOF
    
    # 启动服务器
    ./target/release/game-stream-server -c /tmp/test_server.toml -v &
    SERVER_PID=$!
    
    # 等待服务器启动
    log "等待服务器启动..."
    sleep 3
    
    # 检查服务器是否启动成功
    if ! kill -0 $SERVER_PID 2>/dev/null; then
        log_error "服务器启动失败"
        exit 1
    fi
    
    # 检查 HTTP 端口
    if ! curl -s http://localhost:$SERVER_PORT >/dev/null; then
        log_error "服务器 HTTP 端口无法访问"
        exit 1
    fi
    
    log_success "服务器启动成功 (PID: $SERVER_PID)"
    log_info "HTTP 服务: http://localhost:$SERVER_PORT"
    log_info "RTMP 服务: rtmp://localhost:$RTMP_PORT/live"
}

# 启动客户端
start_client() {
    log "启动推流客户端..."
    
    # 创建临时配置文件
    cat > /tmp/test_client.toml << EOF
[capture]
video_source = "Screen"
audio_source = "System"
fps = 30
width = 1280
height = 720

[encoder]
video_codec = "H264"
audio_codec = "Aac"
video_bitrate = 2000
audio_bitrate = 128

[server]
protocol = "Rtmp"
host = "localhost"
port = $RTMP_PORT
app_name = "live"
stream_key = "$STREAM_KEY"

[network]
buffer_size = 1024
timeout_ms = 5000
retry_count = 3
EOF
    
    # 启动客户端
    ./target/release/game-stream-client -c /tmp/test_client.toml -v &
    CLIENT_PID=$!
    
    # 等待客户端启动
    log "等待客户端启动..."
    sleep 2
    
    # 检查客户端是否启动成功
    if ! kill -0 $CLIENT_PID 2>/dev/null; then
        log_error "客户端启动失败"
        exit 1
    fi
    
    log_success "客户端启动成功 (PID: $CLIENT_PID)"
    log_info "推流地址: rtmp://localhost:$RTMP_PORT/live/$STREAM_KEY"
}

# 测试 FFmpeg 推流
test_ffmpeg_push() {
    log "测试 FFmpeg 推流..."
    
    if ! command -v ffmpeg &> /dev/null; then
        log_warning "FFmpeg 未安装，跳过 FFmpeg 推流测试"
        return
    fi
    
    log "使用 FFmpeg 生成测试视频流..."
    
    # 生成测试视频流（10秒）
    timeout 10 ffmpeg -f lavfi -i testsrc=duration=10:size=1280x720:rate=30 \
        -f lavfi -i sine=frequency=1000:duration=10 \
        -c:v libx264 -preset ultrafast -tune zerolatency \
        -c:a aac -ar 44100 \
        -f flv rtmp://localhost:$RTMP_PORT/live/$STREAM_KEY 2>/dev/null &
    
    FFMPEG_PID=$!
    
    log "FFmpeg 推流进程启动 (PID: $FFMPEG_PID)"
    log_info "推流将持续 10 秒..."
    
    # 等待 FFmpeg 完成
    wait $FFMPEG_PID 2>/dev/null || true
    
    log_success "FFmpeg 推流测试完成"
}

# 显示测试指导
show_test_guide() {
    echo ""
    echo -e "${PURPLE}==================== 测试指导 ====================${NC}"
    echo ""
    echo -e "${GREEN}🌐 网页观看端测试:${NC}"
    echo -e "   1. 打开浏览器访问: ${CYAN}http://localhost:$SERVER_PORT${NC}"
    echo -e "   2. 在流密钥输入框中输入: ${YELLOW}$STREAM_KEY${NC}"
    echo -e "   3. 选择播放协议 (WebRTC 或 HLS)"
    echo -e "   4. 点击 '连接' 按钮开始观看"
    echo ""
    echo -e "${GREEN}📡 API 测试:${NC}"
    echo -e "   • 获取流列表: ${CYAN}curl http://localhost:$SERVER_PORT/api/streams${NC}"
    echo -e "   • 获取流信息: ${CYAN}curl http://localhost:$SERVER_PORT/api/streams/$STREAM_KEY${NC}"
    echo -e "   • 获取统计信息: ${CYAN}curl http://localhost:$SERVER_PORT/api/stats${NC}"
    echo ""
    echo -e "${GREEN}🎥 推流测试:${NC}"
    echo -e "   • RTMP 推流地址: ${CYAN}rtmp://localhost:$RTMP_PORT/live/$STREAM_KEY${NC}"
    echo -e "   • HLS 播放地址: ${CYAN}http://localhost:$SERVER_PORT/hls/$STREAM_KEY/playlist.m3u8${NC}"
    echo ""
    echo -e "${GREEN}🔧 故障排除:${NC}"
    echo -e "   • 查看服务器日志: 观察终端输出"
    echo -e "   • 检查网络连接: 确保防火墙允许端口 $SERVER_PORT 和 $RTMP_PORT"
    echo -e "   • 浏览器开发者工具: F12 查看控制台错误"
    echo ""
    echo -e "${PURPLE}=================================================${NC}"
    echo ""
}

# 等待用户输入
wait_for_user() {
    echo ""
    echo -e "${YELLOW}按 Enter 键继续，或按 Ctrl+C 退出...${NC}"
    read -r
}

# 主函数
main() {
    echo -e "${PURPLE}"
    echo "🎮 游戏直播推流系统 - 完整测试"
    echo "=================================="
    echo -e "${NC}"
    
    # 检查依赖
    check_dependencies
    
    # 启动服务器
    start_server
    
    # 显示测试指导
    show_test_guide
    
    # 等待用户确认
    wait_for_user
    
    # 启动客户端
    start_client
    
    # 测试 FFmpeg 推流
    test_ffmpeg_push
    
    echo ""
    log_success "所有组件已启动完成！"
    echo ""
    echo -e "${GREEN}🎉 测试环境已就绪！${NC}"
    echo -e "   • 服务器运行在: ${CYAN}http://localhost:$SERVER_PORT${NC}"
    echo -e "   • 推流客户端正在运行"
    echo -e "   • 流密钥: ${YELLOW}$STREAM_KEY${NC}"
    echo ""
    echo -e "${YELLOW}按 Ctrl+C 停止所有服务${NC}"
    
    # 保持运行
    while true; do
        sleep 1
        
        # 检查进程是否还在运行
        if ! kill -0 $SERVER_PID 2>/dev/null; then
            log_error "服务器进程意外退出"
            break
        fi
        
        if ! kill -0 $CLIENT_PID 2>/dev/null; then
            log_warning "客户端进程意外退出"
        fi
    done
}

# 运行主函数
main "$@"
