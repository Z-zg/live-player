#!/bin/bash

# FFmpeg 集成启用脚本
# 此脚本将启用项目中的 FFmpeg 支持

set -e

echo "🎬 启用 FFmpeg 支持..."
echo ""

# 检查 FFmpeg 是否已安装
check_ffmpeg() {
    echo "🔍 检查 FFmpeg 安装..."
    
    if ! command -v ffmpeg &> /dev/null; then
        echo "❌ 未找到 FFmpeg，请先安装 FFmpeg"
        echo ""
        echo "安装方法:"
        echo "  macOS: brew install ffmpeg"
        echo "  Ubuntu: sudo apt install ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswscale-dev"
        echo "  Windows: winget install FFmpeg"
        exit 1
    fi
    
    echo "✅ FFmpeg 已安装: $(ffmpeg -version | head -1)"
    echo ""
}

# 检查开发库 (Linux)
check_dev_libs() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "🔍 检查 FFmpeg 开发库..."
        
        if ! pkg-config --exists libavcodec; then
            echo "❌ 缺少 FFmpeg 开发库"
            echo "请安装: sudo apt install libavcodec-dev libavformat-dev libavutil-dev libswscale-dev"
            exit 1
        fi
        
        echo "✅ FFmpeg 开发库已安装"
        echo ""
    fi
}

# 启用 FFmpeg 依赖
enable_ffmpeg_deps() {
    echo "📝 启用 FFmpeg 依赖..."
    
    # game-stream-common
    if grep -q "# ffmpeg-next" game-stream-common/Cargo.toml; then
        sed -i.bak 's/# ffmpeg-next/ffmpeg-next/' game-stream-common/Cargo.toml
        echo "  ✅ 已启用 game-stream-common 中的 FFmpeg"
    fi
    
    # game-stream-client
    if grep -q "# ffmpeg-next" game-stream-client/Cargo.toml; then
        sed -i.bak 's/# ffmpeg-next/ffmpeg-next/' game-stream-client/Cargo.toml
        echo "  ✅ 已启用 game-stream-client 中的 FFmpeg"
    fi
    
    # game-stream-server
    if grep -q "# ffmpeg-next" game-stream-server/Cargo.toml; then
        sed -i.bak 's/# ffmpeg-next/ffmpeg-next/' game-stream-server/Cargo.toml
        echo "  ✅ 已启用 game-stream-server 中的 FFmpeg"
    fi
    
    echo ""
}

# 清理备份文件
cleanup_backups() {
    echo "🧹 清理备份文件..."
    find . -name "*.bak" -delete
    echo ""
}

# 测试编译
test_compilation() {
    echo "🔨 测试编译..."
    
    if cargo check; then
        echo "✅ 编译成功！FFmpeg 支持已启用"
    else
        echo "❌ 编译失败，请检查 FFmpeg 安装"
        echo ""
        echo "常见解决方案:"
        echo "1. 确保 FFmpeg 开发库已安装"
        echo "2. 设置环境变量 FFMPEG_DIR (如果 FFmpeg 安装在非标准位置)"
        echo "3. 在 Linux 上安装 pkg-config"
        exit 1
    fi
    
    echo ""
}

# 主函数
main() {
    echo "开始时间: $(date)"
    echo ""
    
    check_ffmpeg
    check_dev_libs
    enable_ffmpeg_deps
    cleanup_backups
    test_compilation
    
    echo "🎉 FFmpeg 支持已成功启用！"
    echo ""
    echo "现在可以使用以下功能:"
    echo "  - 真实的视频编码 (H.264)"
    echo "  - 真实的音频编码 (AAC)"
    echo "  - 硬件加速编码 (如果支持)"
    echo ""
    echo "重新编译项目:"
    echo "  cargo build --release"
}

# 运行主函数
main "$@"
