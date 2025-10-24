# 多阶段构建Dockerfile for StarryOS Web Demo
FROM node:18-alpine AS frontend-build

# 设置工作目录
WORKDIR /app

# 复制前端文件
COPY web-demo/package*.json ./
COPY web-demo/ ./

# 安装依赖并构建
RUN npm ci --only=production

# 构建前端（如果需要）
# RUN npm run build

# 生产阶段
FROM node:18-alpine

# 安装系统依赖
RUN apk add --no-cache \
    curl \
    wget \
    && rm -rf /var/cache/apk/*

# 创建应用用户
RUN addgroup -g 1001 -S nodejs
RUN adduser -S starryos -u 1001

# 设置工作目录
WORKDIR /app

# 复制前端构建结果
COPY --from=frontend-build --chown=starryos:nodejs /app ./

# 创建API服务器
COPY --chown=starryos:nodejs api-server/ ./api-server/

# 安装API服务器依赖
RUN cd api-server && npm ci --only=production

# 切换到非root用户
USER starryos

# 暴露端口
EXPOSE 8080

# 健康检查
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/health || exit 1

# 启动命令
CMD ["node", "api-server/server.js"]