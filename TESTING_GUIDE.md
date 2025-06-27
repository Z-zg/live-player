# 🧪 游戏直播系统测试指南

## 📋 测试前准备

### 1. 编译项目
```bash
# 确保项目已编译
cargo build --release

# 检查可执行文件
ls -la target/release/game-stream-*
```

### 2. 检查依赖
```bash
# 检查 FFmpeg (可选，用于推流测试)
ffmpeg -version

# 检查端口占用
lsof -i :8080
lsof -i :1935
```

## 🚀 快速测试方法

### 方法1: 一键测试脚本
```bash
# 最简单的测试方式
./scripts/quick-test.sh
```

这个脚本会：
- 自动启动服务器
- 显示测试指导
- 提供推流和观看的具体命令

### 方法2: 完整测试脚本
```bash
# 包含客户端的完整测试
./scripts/test-live-streaming.sh
```

这个脚本会：
- 启动服务器和客户端
- 自动进行 FFmpeg 推流测试
- 提供详细的测试指导

### 方法3: FFmpeg 推流专项测试
```bash
# 专门测试 FFmpeg 推流
./scripts/test-ffmpeg-push.sh
```

## 🔧 手动测试步骤

### 步骤1: 启动服务器
```bash
# 在终端1中启动服务器
./target/release/game-stream-server -v

# 等待看到类似输出:
# [INFO] RTMP server listening on 0.0.0.0:1935
# [INFO] HTTP server listening on 0.0.0.0:8080
```

### 步骤2: 验证服务器
```bash
# 在另一个终端中测试 API
curl http://localhost:8080/api/streams

# 应该返回: {"streams":[]}
```

### 步骤3: 推流测试

#### 选项A: 使用内置客户端
```bash
# 在终端2中启动客户端
./target/release/game-stream-client -v
```

#### 选项B: 使用 FFmpeg 推流
```bash
# 推流测试视频 (30秒)
ffmpeg -f lavfi -i testsrc=duration=30:size=1280x720:rate=30 \
       -f lavfi -i sine=frequency=1000:duration=30 \
       -c:v libx264 -preset ultrafast -tune zerolatency \
       -c:a aac -b:a 128k \
       -f flv rtmp://localhost:1935/live/test_stream_123
```

#### 选项C: 屏幕录制推流 (macOS)
```bash
# 推流屏幕内容
ffmpeg -f avfoundation -i "1:0" \
       -c:v libx264 -preset ultrafast \
       -c:a aac -f flv \
       rtmp://localhost:1935/live/test_stream_123
```

### 步骤4: 观看测试

#### 网页观看端
1. 打开浏览器访问: http://localhost:8080
2. 输入流密钥: `test_stream_123`
3. 选择播放协议:
   - **WebRTC**: 超低延迟 (< 100ms)
   - **HLS**: 兼容性好 (6-30s 延迟)
4. 点击"连接"按钮

#### 测试页面 (调试用)
1. 访问: http://localhost:8080/test.html
2. 查看详细的连接日志
3. 测试 WebRTC 信令交互

#### 直接播放 HLS
```bash
# 使用 ffplay 播放 HLS
ffplay http://localhost:8080/hls/test_stream_123/playlist.m3u8

# 使用 VLC 播放
vlc http://localhost:8080/hls/test_stream_123/playlist.m3u8
```

## 📊 测试验证点

### 1. 服务器功能
- [ ] RTMP 服务器正常启动 (端口 1935)
- [ ] HTTP 服务器正常启动 (端口 8080)
- [ ] WebSocket 信令服务正常
- [ ] API 接口响应正常

### 2. 推流功能
- [ ] RTMP 推流连接成功
- [ ] 流状态正确更新
- [ ] 媒体数据正常接收

### 3. 观看功能
- [ ] 网页界面正常加载
- [ ] WebRTC 连接建立成功
- [ ] HLS 播放列表生成
- [ ] 视频播放正常

### 4. 实时性测试
- [ ] WebRTC 延迟 < 500ms
- [ ] HLS 延迟 < 30s
- [ ] 连接状态实时更新

## 🐛 常见问题排查

### 问题1: 服务器启动失败
```bash
# 检查端口占用
lsof -i :8080
lsof -i :1935

# 杀死占用进程
sudo kill -9 <PID>
```

### 问题2: 推流连接失败
```bash
# 检查 RTMP 服务器状态
curl http://localhost:8080/api/streams

# 查看服务器日志
./target/release/game-stream-server -v
```

### 问题3: WebRTC 连接失败
1. 检查浏览器控制台错误 (F12)
2. 确认 WebSocket 连接正常
3. 检查 STUN 服务器连通性

### 问题4: HLS 播放失败
```bash
# 检查 HLS 文件生成
curl http://localhost:8080/hls/test_stream_123/playlist.m3u8

# 应该返回 M3U8 播放列表
```

### 问题5: FFmpeg 推流失败
```bash
# 检查 FFmpeg 版本
ffmpeg -version

# 使用更简单的推流命令
ffmpeg -re -f lavfi -i testsrc -c:v libx264 -f flv rtmp://localhost:1935/live/test
```

## 📈 性能测试

### 延迟测试
```bash
# 使用时间戳测试延迟
ffmpeg -f lavfi -i "testsrc=size=1280x720:rate=30,drawtext=text='%{localtime}':x=10:y=10:fontsize=48:fontcolor=white" \
       -c:v libx264 -preset ultrafast \
       -f flv rtmp://localhost:1935/live/test_stream_123
```

### 并发测试
```bash
# 启动多个推流 (测试服务器负载)
for i in {1..5}; do
    ffmpeg -f lavfi -i testsrc -c:v libx264 -f flv rtmp://localhost:1935/live/stream_$i &
done
```

### 带宽测试
```bash
# 高码率推流测试
ffmpeg -f lavfi -i testsrc=size=1920x1080:rate=60 \
       -c:v libx264 -b:v 5000k -preset ultrafast \
       -f flv rtmp://localhost:1935/live/test_hd
```

## 📝 测试报告模板

### 基础功能测试
- 服务器启动: ✅/❌
- RTMP 推流: ✅/❌
- WebRTC 观看: ✅/❌
- HLS 观看: ✅/❌
- API 接口: ✅/❌

### 性能测试
- WebRTC 延迟: ___ms
- HLS 延迟: ___s
- 最大并发流: ___个
- CPU 使用率: ___%
- 内存使用: ___MB

### 兼容性测试
- Chrome: ✅/❌
- Firefox: ✅/❌
- Safari: ✅/❌
- Edge: ✅/❌
- 移动端: ✅/❌

## 🎯 测试建议

1. **从简单到复杂**: 先测试基础功能，再测试高级特性
2. **多浏览器测试**: WebRTC 在不同浏览器中表现可能不同
3. **网络环境测试**: 在不同网络条件下测试延迟和稳定性
4. **长时间测试**: 测试系统长时间运行的稳定性
5. **错误恢复测试**: 测试网络断开重连等异常情况

## 📞 获取帮助

如果测试过程中遇到问题:
1. 查看服务器日志输出
2. 检查浏览器开发者工具
3. 参考 README.md 和 FFMPEG_SETUP.md
4. 使用测试页面 (test.html) 进行调试
