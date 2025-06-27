#!/bin/bash

# 性能基准测试脚本
# 测试系统在不同负载下的性能表现

set -e

echo "📊 游戏直播系统性能基准测试"
echo "================================"
echo ""

# 配置参数
RTMP_URL="rtmp://localhost:1935/live"
TEST_DURATION=60  # 测试时长(秒)
CONCURRENT_STREAMS=5  # 并发流数量

# 检查依赖
check_dependencies() {
    local missing_deps=()
    
    if ! command -v ffmpeg &> /dev/null; then
        missing_deps+=("ffmpeg")
    fi
    
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi
    
    if ! command -v htop &> /dev/null && ! command -v top &> /dev/null; then
        missing_deps+=("htop 或 top")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo "❌ 缺少依赖: ${missing_deps[*]}"
        echo "请安装缺少的工具后重试"
        exit 1
    fi
}

# 检查服务器状态
check_server() {
    echo "🔍 检查服务器状态..."
    
    if ! nc -z localhost 1935 2>/dev/null; then
        echo "❌ RTMP 服务器未运行 (端口 1935)"
        return 1
    fi
    
    if ! nc -z localhost 8080 2>/dev/null; then
        echo "❌ HTTP 服务器未运行 (端口 8080)"
        return 1
    fi
    
    echo "✅ 服务器状态正常"
    return 0
}

# 系统信息
show_system_info() {
    echo "💻 系统信息:"
    echo "  操作系统: $(uname -s)"
    echo "  CPU 核心: $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "未知")"
    echo "  内存: $(free -h 2>/dev/null | awk '/^Mem:/ {print $2}' || echo "未知")"
    echo ""
}

# 单流性能测试
test_single_stream() {
    echo "🎯 单流性能测试 (${TEST_DURATION}秒)..."
    
    local stream_key="benchmark_single"
    local log_file="benchmark_single.log"
    
    # 启动推流
    ffmpeg \
        -f lavfi -i "testsrc2=duration=${TEST_DURATION}:size=1920x1080:rate=30" \
        -f lavfi -i "sine=frequency=1000:duration=${TEST_DURATION}" \
        -c:v libx264 -preset fast -tune zerolatency \
        -pix_fmt yuv420p -g 60 -keyint_min 60 \
        -b:v 2500k -maxrate 2500k -bufsize 5000k \
        -c:a aac -b:a 128k -ar 44100 -ac 2 \
        -f flv "${RTMP_URL}/${stream_key}" \
        -y > "$log_file" 2>&1 &
    
    local ffmpeg_pid=$!
    
    # 监控系统资源
    echo "  监控系统资源..."
    local start_time=$(date +%s)
    local max_cpu=0
    local max_memory=0
    
    while kill -0 $ffmpeg_pid 2>/dev/null; do
        # 获取 CPU 和内存使用率
        local cpu_usage=$(top -l 1 -n 0 | grep "CPU usage" | awk '{print $3}' | sed 's/%//' 2>/dev/null || echo "0")
        local memory_usage=$(ps -o pid,ppid,pmem,pcpu,comm -p $ffmpeg_pid 2>/dev/null | tail -1 | awk '{print $3}' || echo "0")
        
        if (( $(echo "$cpu_usage > $max_cpu" | bc -l 2>/dev/null || echo "0") )); then
            max_cpu=$cpu_usage
        fi
        
        if (( $(echo "$memory_usage > $max_memory" | bc -l 2>/dev/null || echo "0") )); then
            max_memory=$memory_usage
        fi
        
        sleep 1
    done
    
    local end_time=$(date +%s)
    local actual_duration=$((end_time - start_time))
    
    echo "  ✅ 单流测试完成"
    echo "    实际时长: ${actual_duration}秒"
    echo "    最大 CPU: ${max_cpu}%"
    echo "    最大内存: ${max_memory}%"
    echo ""
}

# 并发流测试
test_concurrent_streams() {
    echo "🔥 并发流测试 (${CONCURRENT_STREAMS} 个流, ${TEST_DURATION}秒)..."
    
    local pids=()
    local start_time=$(date +%s)
    
    # 启动多个并发流
    for i in $(seq 1 $CONCURRENT_STREAMS); do
        local stream_key="benchmark_concurrent_$i"
        local log_file="benchmark_concurrent_$i.log"
        
        ffmpeg \
            -f lavfi -i "testsrc2=duration=${TEST_DURATION}:size=1280x720:rate=30" \
            -f lavfi -i "sine=frequency=$((1000 + i * 100)):duration=${TEST_DURATION}" \
            -c:v libx264 -preset ultrafast -tune zerolatency \
            -pix_fmt yuv420p -g 60 -keyint_min 60 \
            -b:v 1000k -maxrate 1000k -bufsize 2000k \
            -c:a aac -b:a 64k -ar 44100 -ac 2 \
            -f flv "${RTMP_URL}/${stream_key}" \
            -y > "$log_file" 2>&1 &
        
        pids+=($!)
        echo "  启动流 $i (PID: ${pids[-1]})"
        sleep 0.5  # 避免同时启动造成的冲击
    done
    
    echo "  等待所有流完成..."
    
    # 等待所有流完成
    for pid in "${pids[@]}"; do
        wait $pid
    done
    
    local end_time=$(date +%s)
    local actual_duration=$((end_time - start_time))
    
    echo "  ✅ 并发流测试完成"
    echo "    并发流数: $CONCURRENT_STREAMS"
    echo "    实际时长: ${actual_duration}秒"
    echo ""
}

# 网络延迟测试
test_network_latency() {
    echo "🌐 网络延迟测试..."
    
    # 测试 HTTP API 响应时间
    local api_url="http://localhost:8080/api/streams"
    local response_time=$(curl -o /dev/null -s -w "%{time_total}" "$api_url" 2>/dev/null || echo "0")
    
    echo "  HTTP API 响应时间: ${response_time}秒"
    
    # 测试 WebSocket 连接时间
    local ws_url="ws://localhost:8080/api/webrtc/ws"
    echo "  WebSocket 连接测试: (手动测试)"
    echo ""
}

# 清理测试文件
cleanup() {
    echo "🧹 清理测试文件..."
    rm -f benchmark_*.log
    echo "✅ 清理完成"
}

# 主函数
main() {
    echo "开始时间: $(date)"
    echo ""
    
    check_dependencies
    show_system_info
    
    if ! check_server; then
        echo ""
        echo "请先启动服务器:"
        echo "  cargo run --bin game-stream-server"
        exit 1
    fi
    
    echo ""
    
    # 运行测试
    test_single_stream
    test_concurrent_streams
    test_network_latency
    
    echo "📈 基准测试完成!"
    echo "结束时间: $(date)"
    echo ""
    echo "💡 优化建议:"
    echo "  - 如果 CPU 使用率过高，考虑降低编码质量或启用硬件加速"
    echo "  - 如果内存使用率过高，考虑调整缓冲区大小"
    echo "  - 如果网络延迟过高，检查网络配置和服务器性能"
    
    cleanup
}

# 捕获中断信号
trap 'echo ""; echo "🛑 测试被中断"; cleanup; exit 1' INT

# 运行主函数
main "$@"
