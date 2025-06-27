# FFmpeg 安装和配置指南

本项目需要 FFmpeg 来进行音视频编解码。由于 FFmpeg 的 Rust 绑定配置较为复杂，这里提供详细的安装和配置指南。

## 🔧 系统要求

- **FFmpeg**: 4.0 或更高版本
- **开发库**: 需要 FFmpeg 的开发头文件和库文件

## 📦 安装 FFmpeg

### Windows

#### 方法 1: 使用 winget (推荐)
```bash
winget install FFmpeg
```

#### 方法 2: 使用 Chocolatey
```bash
choco install ffmpeg
```

#### 方法 3: 手动安装
1. 访问 [FFmpeg 官网](https://ffmpeg.org/download.html#build-windows)
2. 下载预编译的二进制文件
3. 解压到 `C:\ffmpeg`
4. 将 `C:\ffmpeg\bin` 添加到系统 PATH

### macOS

#### 使用 Homebrew (推荐)
```bash
brew install ffmpeg
```

#### 使用 MacPorts
```bash
sudo port install ffmpeg
```

### Linux

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswscale-dev libavfilter-dev libavdevice-dev
```

#### CentOS/RHEL/Fedora
```bash
# CentOS/RHEL (需要 EPEL 仓库)
sudo yum install epel-release
sudo yum install ffmpeg-devel

# Fedora
sudo dnf install ffmpeg-devel
```

#### Arch Linux
```bash
sudo pacman -S ffmpeg
```

## 🔍 验证安装

安装完成后，验证 FFmpeg 是否正确安装：

```bash
# 检查 FFmpeg 版本
ffmpeg -version

# 检查开发库 (Linux)
pkg-config --libs libavcodec
pkg-config --libs libavformat
```

## 🛠️ Rust 项目配置

### 启用 FFmpeg 支持

1. **取消注释 FFmpeg 依赖**

编辑 `game-stream-common/Cargo.toml`：
```toml
# 取消注释这一行
ffmpeg-next = "7.0"
```

编辑 `game-stream-client/Cargo.toml`：
```toml
# 取消注释这一行
ffmpeg-next = "7.0"
```

编辑 `game-stream-server/Cargo.toml`：
```toml
# 取消注释这一行
ffmpeg-next = "7.0"
```

2. **设置环境变量 (如果需要)**

如果 FFmpeg 安装在非标准位置，可能需要设置环境变量：

```bash
# Linux/macOS
export FFMPEG_DIR=/path/to/ffmpeg
export PKG_CONFIG_PATH=/path/to/ffmpeg/lib/pkgconfig

# Windows (PowerShell)
$env:FFMPEG_DIR = "C:\ffmpeg"
```

3. **重新编译项目**

```bash
cargo clean
cargo build
```

## 🚨 常见问题

### 问题 1: 找不到 FFmpeg 库

**错误信息**:
```
error: failed to run custom build command for `ffmpeg-sys`
```

**解决方案**:
1. 确保 FFmpeg 已正确安装
2. 检查 PATH 环境变量
3. 在 Linux 上安装开发包 (`-dev` 或 `-devel`)

### 问题 2: 版本不兼容

**错误信息**:
```
error: FFmpeg version X.X is not supported
```

**解决方案**:
1. 升级 FFmpeg 到 4.0 或更高版本
2. 或者降级 `ffmpeg-next` 依赖版本

### 问题 3: Windows 上的链接错误

**解决方案**:
1. 确保安装了完整的 FFmpeg 包（包含开发库）
2. 使用 MSYS2 安装 FFmpeg：
   ```bash
   pacman -S mingw-w64-x86_64-ffmpeg
   ```

### 问题 4: macOS 上的权限问题

**解决方案**:
1. 使用 Homebrew 重新安装：
   ```bash
   brew uninstall ffmpeg
   brew install ffmpeg
   ```

## 🔄 替代方案

如果 FFmpeg 配置仍有问题，可以使用以下替代方案：

### 1. 使用 Docker

创建 `Dockerfile`：
```dockerfile
FROM rust:1.70

# 安装 FFmpeg
RUN apt-get update && apt-get install -y \
    ffmpeg \
    libavcodec-dev \
    libavformat-dev \
    libavutil-dev \
    libswscale-dev

WORKDIR /app
COPY . .
RUN cargo build --release
```

### 2. 使用预编译的二进制文件

暂时注释掉 FFmpeg 相关代码，使用模拟的编码器：

```rust
// 在 codec.rs 中使用模拟实现
impl VideoEncoder for H264Encoder {
    fn encode_frame(&mut self, frame: &VideoFrame) -> StreamResult<Vec<EncodedPacket>> {
        // 模拟编码逻辑
        // 实际项目中需要替换为真实的 FFmpeg 调用
    }
}
```

## 📚 进一步阅读

- [FFmpeg 官方文档](https://ffmpeg.org/documentation.html)
- [ffmpeg-next Rust 库文档](https://docs.rs/ffmpeg-next/)
- [Rust 音视频处理指南](https://github.com/zmwangx/rust-ffmpeg)

## 💡 提示

1. **开发环境**: 建议在开发时使用 Docker 来避免环境配置问题
2. **生产环境**: 确保所有依赖都正确安装和配置
3. **性能优化**: 考虑启用硬件加速 (NVENC, QuickSync 等)
4. **跨平台**: 不同平台的 FFmpeg 配置可能有差异，需要分别测试

---

如果遇到其他问题，请查看项目的 [Issues](https://github.com/your-username/live-player/issues) 或创建新的 Issue。
