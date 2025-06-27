# 🎮 游戏直播推流系统

一个基于 Rust 的高性能游戏直播推流系统，支持多种推流协议和观看方式，专为低延迟游戏直播设计。

## ✨ 特性

### 🚀 高性能架构
- **纯 Rust 实现**：充分利用 Rust 的内存安全和并发优势
- **异步 I/O**：基于 tokio 异步运行时，支持高并发连接
- **零拷贝优化**：最小化内存拷贝，提升性能

### 📡 多协议支持
- **推流协议**：
  - ✅ RTMP (Real-Time Messaging Protocol)
  - 🔄 SRT (Secure Reliable Transport) - 规划中
  - 🔄 自定义 UDP/TCP 协议 - 规划中
- **观看协议**：
  - ✅ WebRTC (超低延迟)
  - ✅ HLS (HTTP Live Streaming)
  - 🔄 DASH (Dynamic Adaptive Streaming) - 规划中
  - ✅ RTMP 转发

### 🎥 强大的媒体处理
- **视频编码**：H.264 (支持硬件加速)
- **音频编码**：AAC
- **屏幕捕获**：支持全屏、窗口、区域捕获
- **音频捕获**：系统音频、麦克风输入

### 🌐 现代化 Web 界面
- **响应式设计**：支持桌面和移动设备
- **实时统计**：观看者数量、延迟、码率监控
- **多协议切换**：WebRTC 和 HLS 无缝切换

## 🏗️ 系统架构

```
┌─────────────────┐    RTMP/SRT     ┌─────────────────┐
│  游戏直播客户端   │ ──────────────► │   流媒体服务器    │
│                │                │                │
│ • 屏幕/音频捕获  │                │ • 多协议接收      │
│ • H.264/AAC编码 │                │ • 流管理与分发    │
│ • 多协议推流     │                │ • WebRTC信令     │
└─────────────────┘                │ • HLS切片生成    │
                                   └─────────────────┘
                                            │
                                            ▼
                                   ┌─────────────────┐
                                   │   网页观看端     │
                                   │                │
                                   │ • WebRTC播放    │
                                   │ • HLS播放       │
                                   │ • 实时统计      │
                                   └─────────────────┘
```

## 🚀 快速开始

### 环境要求

- **Rust**: 1.70+
- **FFmpeg**: 4.0+ (用于音视频编解码)
- **操作系统**: Windows 10+, macOS 10.15+, Linux (Ubuntu 20.04+)

### 安装依赖

#### Windows
```bash
# 安装 FFmpeg
winget install FFmpeg

# 或使用 Chocolatey
choco install ffmpeg
```

#### macOS
```bash
# 使用 Homebrew
brew install ffmpeg
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswscale-dev
```

### 编译项目

```bash
# 克隆项目
git clone https://github.com/your-username/live-player.git
cd live-player

# 编译所有组件
cargo build --release

# 或分别编译
cargo build --release --bin game-stream-server
cargo build --release --bin game-stream-client
```

### 运行服务器

```bash
# 使用默认配置运行
./target/release/game-stream-server

# 或指定配置文件
./target/release/game-stream-server --config server.toml

# 指定端口
./target/release/game-stream-server --rtmp-port 1935 --http-port 8080
```

服务器启动后将监听：
- **RTMP**: `rtmp://localhost:1935/live`
- **HTTP/WebRTC**: `http://localhost:8080`

### 运行客户端

```bash
# 使用默认配置推流
./target/release/game-stream-client

# 指定流密钥和服务器
./target/release/game-stream-client --stream-key my_stream --host localhost

# 使用配置文件
./target/release/game-stream-client --config client.toml
```

## 🧪 快速测试

### 一键测试脚本

```bash
# 快速启动服务器测试
./scripts/quick-test.sh

# 完整的直播测试 (包含客户端)
./scripts/test-live-streaming.sh

# FFmpeg 推流测试
./scripts/test-ffmpeg-push.sh
```

### 手动测试步骤

1. **启动服务器**
   ```bash
   ./target/release/game-stream-server
   ```

2. **推流测试** (选择一种方式)
   ```bash
   # 方式1: 使用内置客户端
   ./target/release/game-stream-client

   # 方式2: 使用 FFmpeg 推流
   ffmpeg -f lavfi -i testsrc=size=1280x720:rate=30 \
          -f lavfi -i sine=frequency=1000 \
          -c:v libx264 -preset ultrafast \
          -c:a aac -f flv \
          rtmp://localhost:1935/live/test_stream_123
   ```

3. **观看测试**
   - 网页观看端: http://localhost:8080
   - 测试页面: http://localhost:8080/test.html
   - 流密钥: `test_stream_123`

4. **API 测试**
   ```bash
   # 获取流列表
   curl http://localhost:8080/api/streams

   # 获取流信息
   curl http://localhost:8080/api/streams/test_stream_123

   # 获取统计信息
   curl http://localhost:8080/api/stats
   ```

### 观看直播

1. **Web 观看端**：打开浏览器访问 `http://localhost:8080`
2. **输入流密钥**：默认为 `test_stream`
3. **选择协议**：WebRTC (低延迟) 或 HLS (兼容性好)
4. **点击连接**：开始观看直播

## ⚙️ 配置说明

### 客户端配置 (client.toml)

```toml
[server]
protocol = "Rtmp"  # 推流协议
host = "localhost"
port = 1935
stream_key = "test_stream"

[encoding.video]
codec = "H264"
width = 1920
height = 1080
fps = 30
bitrate = 2500  # kbps

[capture.video_source]
Screen = { display_index = 0 }  # 捕获主显示器
```

### 服务器配置 (server.toml)

```toml
[rtmp]
bind_addr = "0.0.0.0"
port = 1935

[http]
bind_addr = "0.0.0.0"
port = 8080

[auth]
enabled = false
valid_stream_keys = ["test_stream"]
```

## 🧪 测试指南

### 1. 基础功能测试

```bash
# 终端 1: 启动服务器
cargo run --bin game-stream-server

# 终端 2: 启动客户端
cargo run --bin game-stream-client

# 浏览器: 访问 http://localhost:8080
```

### 2. 性能测试

```bash
# 启用详细日志
RUST_LOG=debug cargo run --bin game-stream-server

# 监控系统资源
htop  # Linux/macOS
# 或任务管理器 (Windows)
```

### 3. 网络测试

```bash
# 测试 RTMP 连接
ffmpeg -f lavfi -i testsrc=duration=10:size=1280x720:rate=30 \
       -f lavfi -i sine=frequency=1000:duration=10 \
       -c:v libx264 -c:a aac \
       -f flv rtmp://localhost:1935/live/test_stream

# 测试 HLS 播放
curl http://localhost:8080/hls/test_stream/playlist.m3u8
```

## 📊 性能指标

### 延迟对比
- **WebRTC**: < 100ms (超低延迟)
- **HLS**: 6-30s (标准延迟)
- **RTMP**: 2-5s (中等延迟)

### 系统要求
- **CPU**: 推荐 4 核心以上 (支持硬件编码可降低要求)
- **内存**: 最少 4GB，推荐 8GB+
- **网络**: 上行带宽 > 推流码率 × 1.2

### 并发能力
- **推流**: 支持 100+ 并发推流
- **观看**: 支持 1000+ 并发观看 (取决于服务器配置)

## 🔧 故障排除

### 常见问题

1. **编译失败**
   ```bash
   # 检查 Rust 版本
   rustc --version

   # 更新 Rust
   rustup update

   # 清理并重新编译
   cargo clean && cargo build
   ```

2. **FFmpeg 相关错误**
   ```bash
   # 检查 FFmpeg 安装
   ffmpeg -version

   # 检查开发库 (Linux)
   pkg-config --libs libavcodec
   ```

3. **网络连接问题**
   ```bash
   # 检查端口占用
   netstat -tulpn | grep :1935
   netstat -tulpn | grep :8080

   # 检查防火墙设置
   sudo ufw status  # Linux
   ```

4. **WebRTC 连接失败**
   - 检查浏览器控制台错误
   - 确认 HTTPS 或 localhost 环境
   - 检查 ICE 服务器配置

### 日志分析

```bash
# 启用详细日志
RUST_LOG=game_stream_server=debug,game_stream_client=debug cargo run

# 日志级别说明
# ERROR: 错误信息
# WARN:  警告信息
# INFO:  一般信息
# DEBUG: 调试信息
```

## 🛣️ 发展路线图

### v0.2.0 (计划中)
- [ ] SRT 协议支持
- [ ] DASH 流媒体支持
- [ ] 硬件编码优化 (NVENC, QuickSync)
- [ ] 集群部署支持

### v0.3.0 (计划中)
- [ ] 录制功能
- [ ] 转码服务
- [ ] CDN 集成
- [ ] 管理后台

### v1.0.0 (长期目标)
- [ ] 生产环境优化
- [ ] 完整的监控系统
- [ ] 自动扩缩容
- [ ] 商业化功能

## 🤝 贡献指南

我们欢迎各种形式的贡献！

### 开发环境设置

```bash
# 克隆项目
git clone https://github.com/your-username/live-player.git
cd live-player

# 安装开发依赖
cargo install cargo-watch cargo-audit

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

### 提交规范

- 使用清晰的提交信息
- 遵循 Rust 代码规范
- 添加必要的测试
- 更新相关文档

## 📄 许可证

本项目采用 [MIT 许可证](LICENSE)。

## 🙏 致谢

- [tokio](https://tokio.rs/) - 异步运行时
- [axum](https://github.com/tokio-rs/axum) - Web 框架
- [webrtc-rs](https://github.com/webrtc-rs/webrtc) - WebRTC 实现
- [FFmpeg](https://ffmpeg.org/) - 音视频处理

## 📞 联系我们

- **Issues**: [GitHub Issues](https://github.com/your-username/live-player/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-username/live-player/discussions)
---

⭐ 如果这个项目对您有帮助，请给我们一个 Star！
