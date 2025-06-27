# 游戏直播推流系统 Makefile

.PHONY: help build build-release clean test run-server run-client dev benchmark install-deps format lint check

# 默认目标
help:
	@echo "🎮 游戏直播推流系统"
	@echo ""
	@echo "可用命令:"
	@echo "  build         - 编译项目 (debug 模式)"
	@echo "  build-release - 编译项目 (release 模式)"
	@echo "  clean         - 清理编译文件"
	@echo "  test          - 运行测试"
	@echo "  run-server    - 运行服务器"
	@echo "  run-client    - 运行客户端"
	@echo "  dev           - 启动开发环境"
	@echo "  benchmark     - 运行性能测试"
	@echo "  install-deps  - 安装系统依赖"
	@echo "  format        - 格式化代码"
	@echo "  lint          - 代码检查"
	@echo "  check         - 检查代码 (不编译)"

# 编译项目
build:
	@echo "🔨 编译项目 (debug 模式)..."
	cargo build

build-release:
	@echo "🔨 编译项目 (release 模式)..."
	cargo build --release

# 清理
clean:
	@echo "🧹 清理编译文件..."
	cargo clean
	rm -rf hls/ dash/ *.log

# 测试
test:
	@echo "🧪 运行测试..."
	cargo test

# 运行服务器
run-server:
	@echo "🖥️  启动流媒体服务器..."
	RUST_LOG=info cargo run --bin game-stream-server

# 运行客户端
run-client:
	@echo "📹 启动推流客户端..."
	RUST_LOG=info cargo run --bin game-stream-client

# 开发环境
dev:
	@echo "🚀 启动开发环境..."
	./scripts/start-dev.sh

# 性能测试
benchmark:
	@echo "📊 运行性能测试..."
	./scripts/benchmark.sh

# 安装系统依赖
install-deps:
	@echo "📦 检查并安装系统依赖..."
	@if command -v brew >/dev/null 2>&1; then \
		echo "使用 Homebrew 安装依赖..."; \
		brew install ffmpeg; \
	elif command -v apt >/dev/null 2>&1; then \
		echo "使用 apt 安装依赖..."; \
		sudo apt update && sudo apt install -y ffmpeg libavcodec-dev libavformat-dev libavutil-dev libswscale-dev; \
	elif command -v yum >/dev/null 2>&1; then \
		echo "使用 yum 安装依赖..."; \
		sudo yum install -y ffmpeg-devel; \
	else \
		echo "❌ 未识别的包管理器，请手动安装 FFmpeg"; \
		exit 1; \
	fi

# 代码格式化
format:
	@echo "✨ 格式化代码..."
	cargo fmt

# 代码检查
lint:
	@echo "🔍 代码检查..."
	cargo clippy -- -D warnings

# 检查代码 (不编译)
check:
	@echo "🔍 检查代码..."
	cargo check

# 创建必要的目录
setup-dirs:
	@echo "📁 创建必要的目录..."
	mkdir -p hls dash web

# 完整的构建流程
all: setup-dirs build test
	@echo "✅ 构建完成!"

# 发布准备
release-prep: clean format lint test build-release
	@echo "🚀 发布准备完成!"
	@echo ""
	@echo "编译产物位置:"
	@echo "  服务器: ./target/release/game-stream-server"
	@echo "  客户端: ./target/release/game-stream-client"

# Docker 相关 (未来扩展)
docker-build:
	@echo "🐳 构建 Docker 镜像..."
	docker build -t game-stream-server .

docker-run:
	@echo "🐳 运行 Docker 容器..."
	docker run -p 1935:1935 -p 8080:8080 game-stream-server

# 文档生成
docs:
	@echo "📚 生成文档..."
	cargo doc --open

# 安全审计
audit:
	@echo "🔒 安全审计..."
	cargo audit

# 更新依赖
update:
	@echo "⬆️  更新依赖..."
	cargo update
