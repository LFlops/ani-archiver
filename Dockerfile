# === 第一阶段：构建 ===
FROM rust:1.70 AS builder

# 设置工作目录
WORKDIR /app

# 复制 Cargo.toml 和 Cargo.lock，并下载依赖
COPY Cargo.toml Cargo.lock ./
RUN mkdir src/ && echo 'fn main() {}' > src/main.rs && cargo build --release
RUN rm -rf ./target/release/ani-archiver src/

# 复制整个项目代码
COPY . .

# 编译应用
# 使用 --locked 确保构建的可复现性
RUN cargo build --release --locked

# === 第二阶段：创建最终镜像 ===
# 使用一个更小的基础镜像，例如 Debian slim 或 Alpine
FROM debian:bullseye-slim

# 安装必要的运行时依赖
# 例如，如果你的应用使用了 openssl，可能需要安装 libssl-dev
RUN apt-get update && apt-get install -y openssl libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /usr/local/bin

# 从构建阶段复制编译好的可执行文件
COPY --from=builder /app/target/release/ani-archiver .

# 暴露容器端口 (如果你的应用是Web服务)
EXPOSE 8000

# 定义容器启动命令
CMD ["./ani-archiver"]