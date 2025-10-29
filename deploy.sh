#!/bin/bash

# ================================================
# Kubernetes 部署脚本
# ================================================

# 镜像名称和版本
IMAGE_NAME="your-dockerhub-username/ani-archiver"
IMAGE_TAG="v1.0.0"

# K8s 配置资源名称
CONFIGMAP_NAME="ani-archiver-config"
SECRET_NAME="ani-archiver-secret"

# 本地配置文件路径
CONFIG_FILE="./app-config.env"
SECRET_FILE="./app-secret.env"

# --- 1. 构建 Docker 镜像 ---
echo "Building Docker image: ${IMAGE_NAME}:${IMAGE_TAG}..."
docker build -t "${IMAGE_NAME}:${IMAGE_TAG}" .

if [ $? -ne 0 ]; then
    echo "Docker build failed!"
    exit 1
fi

# --- 2. 推送镜像到仓库 ---
echo "Pushing image to Docker Hub..."
docker push "${IMAGE_NAME}:${IMAGE_TAG}"

if [ $? -ne 0 ]; then
    echo "Docker push failed!"
    exit 1
fi

# --- 3. 创建或更新 K8s 配置 ---
echo "Applying Kubernetes configurations..."

# 如果 .env 文件存在，则创建或更新 ConfigMap 和 Secret
if [ -f "${CONFIG_FILE}" ]; then
    kubectl create configmap "${CONFIGMAP_NAME}" --from-env-file="${CONFIG_FILE}" --dry-run=client -o yaml | kubectl apply -f -
    echo "ConfigMap '${CONFIGMAP_NAME}' applied."
else
    echo "Warning: '${CONFIG_FILE}' not found. Skipping ConfigMap creation."
fi

if [ -f "${SECRET_FILE}" ]; then
    kubectl create secret generic "${SECRET_NAME}" --from-env-file="${SECRET_FILE}" --dry-run=client -o yaml | kubectl apply -f -
    echo "Secret '${SECRET_NAME}' applied."
else
    echo "Warning: '${SECRET_FILE}' not found. Skipping Secret creation."
fi

# --- 4. 应用 K8s Deployment 和 Service ---
echo "Applying Kubernetes deployment files..."
# 替换 Deployment 文件中的镜像名称
sed "s|IMAGE_PLACEHOLDER|${IMAGE_NAME}:${IMAGE_TAG}|g" ./deployment.yaml | kubectl apply -f -

echo "Deployment completed successfully!"