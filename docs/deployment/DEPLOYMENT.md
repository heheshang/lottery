# 🚀 部署指南

> **文档版本**: v1.0.0  
> **创建日期**: 2024-08-15  
> **最后更新**: 2024-08-15  

## 📋 目录

1. [部署概述](#部署概述)
2. [本地构建](#本地构建)
3. [多平台构建](#多平台构建)
4. [CI/CD部署](#cicd部署)
5. [容器化部署](#容器化部署)
6. [云服务部署](#云服务部署)
7. [监控与运维](#监控与运维)
8. [回滚策略](#回滚策略)

---

## 部署概述

智能抽奖系统支持多种部署方式，包括本地构建、CI/CD自动化、容器化部署和云服务部署。

### 部署架构
```
本地开发 → 测试环境 → 预发布环境 → 生产环境
```

### 部署方式对比
| 方式 | 适用场景 | 优点 | 缺点 |
|---|---|---|---|
| **本地构建** | 开发测试 | 快速迭代 | 手动操作 |
| **CI/CD** | 生产发布 | 自动化 | 配置复杂 |
| **容器化** | 云部署 | 一致性 | 资源占用 |
| **云服务** | 大规模 | 弹性扩展 | 成本较高 |

---

## 本地构建

### 环境准备
```bash
# 确认环境
node --version    # ≥18.0.0
pnpm --version    # ≥8.0.0
rustc --version   # ≥1.75.0
cargo --version   # 最新

# 安装系统依赖
# Windows: Visual Studio Build Tools
# macOS: Xcode Command Line Tools
# Linux: build-essential libwebkit2gtk-4.0-dev
```

### 构建步骤

#### 1. 清理环境
```bash
# 清理构建缓存
pnpm clean
cargo clean

# 清理旧版本
rm -rf dist/
rm -rf src-tauri/target/release/
```

#### 2. 构建前端
```bash
# 构建前端
pnpm build

# 验证构建结果
ls -la dist/
```

#### 3. 构建后端
```bash
# 构建Rust后端
cd src-tauri
cargo build --release

# 验证构建结果
ls -la target/release/
```

#### 4. 打包应用
```bash
# 完整构建
pnpm tauri build

# 查看构建结果
ls -la src-tauri/target/release/bundle/
```

---

## 多平台构建

### 平台特定配置

#### Windows构建
```bash
# Windows构建命令
pnpm tauri build --target x86_64-pc-windows-msvc

# 输出文件
src-tauri/target/release/bundle/msi/smart-lottery_1.0.0_x64.msi
src-tauri/target/release/bundle/nsis/smart-lottery_1.0.0_x64-setup.exe
```

#### macOS构建
```bash
# macOS构建命令
pnpm tauri build --target x86_64-apple-darwin
pnpm tauri build --target aarch64-apple-darwin

# 输出文件
src-tauri/target/release/bundle/dmg/smart-lottery_1.0.0_x64.dmg
src-tauri/target/release/bundle/macos/smart-lottery.app
```

#### Linux构建
```bash
# Linux构建命令
pnpm tauri build --target x86_64-unknown-linux-gnu

# 输出文件
src-tauri/target/release/bundle/deb/smart-lottery_1.0.0_amd64.deb
src-tauri/target/release/bundle/rpm/smart-lottery-1.0.0-1.x86_64.rpm
src-tauri/target/release/bundle/appimage/smart-lottery_1.0.0_amd64.AppImage
```

### 构建配置优化

#### Cargo.toml优化
```toml
[profile.release]
codegen-units = 1
lto = true
opt-level = "z"
panic = "abort"
strip = true

[package.metadata.bundle]
identifier = "com.smartlottery.app"
category = "Productivity"
short_description = "AI-powered lottery system"
long_description = "A modern desktop application for intelligent lottery management with AI capabilities"
```

---

## CI/CD部署

### GitHub Actions配置

#### 基本工作流
```yaml
# .github/workflows/build.yml
name: Build and Release

on:
  push:
    tags: ['v*']
  pull_request:
    branches: [main, develop]

jobs:
  build:
    strategy:
      matrix:
        include:
          - platform: 'macos-latest'
            args: '--target x86_64-apple-darwin'
          - platform: 'macos-latest'
            args: '--target aarch64-apple-darwin'
          - platform: 'ubuntu-22.04'
            args: ''
          - platform: 'windows-latest'
            args: ''

    runs-on: ${{ matrix.platform }}

    steps:
      - name: Checkout repository
        uses: actions/checkout@v4

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '18'
          cache: 'pnpm'

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install dependencies
        run: |
          pnpm install
          cargo install cargo-audit
          cargo install cargo-deny

      - name: Run tests
        run: |
          pnpm test
          cargo test
          cargo audit
          cargo deny check

      - name: Build application
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: '智能抽奖系统 v${{ github.ref_name }}'
          releaseBody: 'See the assets to download this version'
          releaseDraft: true
          prerelease: false
```

#### 多环境部署
```yaml
# .github/workflows/deploy.yml
name: Deploy to Environments

on:
  push:
    branches: [main, develop]

jobs:
  deploy-dev:
    if: github.ref == 'refs/heads/develop'
    runs-on: ubuntu-latest
    environment: development
    steps:
      - name: Deploy to Dev
        run: |
          echo "Deploying to development environment"
          # 部署逻辑

  deploy-staging:
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - name: Deploy to Staging
        run: |
          echo "Deploying to staging environment"
          # 部署逻辑

  deploy-production:
    if: startsWith(github.ref, 'refs/tags/v')
    runs-on: ubuntu-latest
    environment: production
    steps:
      - name: Deploy to Production
        run: |
          echo "Deploying to production environment"
          # 部署逻辑
```

---

## 容器化部署

### Docker配置

#### Dockerfile
```dockerfile
# Dockerfile
FROM node:18-alpine AS frontend
WORKDIR /app
COPY package*.json ./
RUN pnpm install --frozen-lockfile
COPY . .
RUN pnpm build

FROM rust:1.75-alpine AS backend
WORKDIR /app
COPY src-tauri ./src-tauri
RUN cd src-tauri && cargo build --release

FROM alpine:latest
RUN apk add --no-cache libwebkit2gtk-4.0-dev
COPY --from=frontend /app/dist /app/dist
COPY --from=backend /app/src-tauri/target/release/smart-lottery /usr/local/bin/
CMD ["smart-lottery"]
```

#### Docker Compose
```yaml
# docker-compose.yml
version: '3.8'
services:
  smart-lottery:
    build: .
    ports:
      - "1420:1420"
    environment:
      - NODE_ENV=production
    volumes:
      - ./data:/app/data
    restart: unless-stopped

  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - smart-lottery
```

### Kubernetes部署

#### Deployment配置
```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: smart-lottery
  labels:
    app: smart-lottery
spec:
  replicas: 3
  selector:
    matchLabels:
      app: smart-lottery
  template:
    metadata:
      labels:
        app: smart-lottery
    spec:
      containers:
      - name: smart-lottery
        image: smartlottery/app:1.0.0
        ports:
        - containerPort: 1420
        env:
        - name: NODE_ENV
          value: "production"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
```

#### Service配置
```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: smart-lottery-service
spec:
  selector:
    app: smart-lottery
  ports:
    - protocol: TCP
      port: 80
      targetPort: 1420
  type: LoadBalancer
```

---

## 云服务部署

### AWS部署

#### ECS配置
```json
// task-definition.json
{
  "family": "smart-lottery",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "256",
  "memory": "512",
  "containerDefinitions": [
    {
      "name": "smart-lottery",
      "image": "your-account.dkr.ecr.region.amazonaws.com/smart-lottery:1.0.0",
      "portMappings": [
        {
          "containerPort": 1420,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "NODE_ENV",
          "value": "production"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/ecs/smart-lottery",
          "awslogs-region": "us-west-2",
          "awslogs-stream-prefix": "ecs"
        }
      }
    }
  ]
}
```

#### CloudFormation模板
```yaml
# cloudformation.yaml
AWSTemplateFormatVersion: '2010-09-09'
Description: 'Smart Lottery System Infrastructure'

Resources:
  ECSCluster:
    Type: AWS::ECS::Cluster
    Properties:
      ClusterName: smart-lottery-cluster

  ApplicationLoadBalancer:
    Type: AWS::ElasticLoadBalancingV2::LoadBalancer
    Properties:
      Name: smart-lottery-alb
      Scheme: internet-facing
      Type: application
      SecurityGroups:
        - !Ref LoadBalancerSecurityGroup
      Subnets:
        - subnet-12345678
        - subnet-87654321

  LoadBalancerSecurityGroup:
    Type: AWS::EC2::SecurityGroup
    Properties:
      GroupDescription: Security group for load balancer
      SecurityGroupIngress:
        - IpProtocol: tcp
          FromPort: 80
          ToPort: 80
          CidrIp: 0.0.0.0/0
```

### GCP部署

#### Cloud Run配置
```yaml
# cloud-run.yaml
apiVersion: serving.knative.dev/v1
kind: Service
metadata:
  name: smart-lottery
  namespace: default
spec:
  template:
    metadata:
      annotations:
        autoscaling.knative.dev/minScale: "1"
        autoscaling.knative.dev/maxScale: "10"
    spec:
      containers:
      - image: gcr.io/your-project/smart-lottery:1.0.0
        ports:
        - containerPort: 1420
        env:
        - name: NODE_ENV
          value: production
        resources:
          limits:
            cpu: "500m"
            memory: "512Mi"
          requests:
            cpu: "250m"
            memory: "256Mi"
```

### Azure部署

#### Azure Container Instances
```bash
# Azure CLI部署
az container create \
  --resource-group smart-lottery-rg \
  --name smart-lottery \
  --image your-registry.azurecr.io/smart-lottery:1.0.0 \
  --ports 1420 \
  --cpu 1 \
  --memory 1 \
  --environment-variables NODE_ENV=production
```

---

## 监控与运维

### 监控指标

#### 应用指标
- **启动时间**: < 2秒
- **内存使用**: < 500MB
- **CPU使用**: < 10%
- **错误率**: < 0.1%

#### 业务指标
- **抽奖次数**: 每分钟
- **用户活跃度**: 日活跃/月活跃
- **响应时间**: < 500ms
- **成功率**: > 99.9%

### 日志监控

#### 日志配置
```yaml
# config/logging.yml
logging:
  level: INFO
  appenders:
    - type: console
    - type: file
      path: /var/log/smart-lottery/app.log
      max_files: 10
      max_size: 100MB
```

#### 监控工具
```bash
# 使用Prometheus + Grafana
docker-compose -f monitoring.yml up -d

# 查看应用日志
docker logs smart-lottery
kubectl logs -f deployment/smart-lottery
```

### 健康检查

#### 健康检查端点
```typescript
// 健康检查API
app.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    version: '1.0.0',
    uptime: process.uptime()
  });
});
```

---

## 回滚策略

### 版本回滚

#### Git回滚
```bash
# 紧急回滚
git checkout v0.9.9
git checkout -b hotfix/rollback
git push origin hotfix/rollback

# 创建回滚版本
git revert HEAD
```

#### Docker回滚
```bash
# 回滚到上一个版本
docker service rollback smart-lottery

# 使用特定版本
docker run smartlottery/app:0.9.9
```

### 数据库回滚

#### 备份策略
```bash
# 自动备份脚本
#!/bin/bash
BACKUP_DIR="/backups/smart-lottery"
DATE=$(date +%Y%m%d_%H%M%S)

cp -r /app/data "$BACKUP_DIR/data_$DATE"
gzip "$BACKUP_DIR/data_$DATE"

# 清理旧备份
find "$BACKUP_DIR" -name "data_*.gz" -mtime +7 -delete
```

#### 恢复策略
```bash
# 从备份恢复
cp /backups/smart-lottery/data_20240815_120000.gz /app/data/
gunzip /app/data/data_20240815_120000.gz
```

---

## 📊 部署检查清单

### 预部署检查
- [ ] 所有测试通过
- [ ] 安全扫描完成
- [ ] 性能测试通过
- [ ] 文档已更新
- [ ] 备份策略就绪
- [ ] 监控配置完成
- [ ] 回滚方案准备

### 部署后验证
- [ ] 应用正常启动
- [ ] 功能测试通过
- [ ] 性能指标正常
- [ ] 日志无错误
- [ ] 监控数据正常
- [ ] 用户反馈良好

---

## 📞 部署支持

### 联系方式
- **部署问题**: [GitHub Issues](https://github.com/your-org/smart-lottery/issues)
- **紧急支持**: deploy-support@smartlottery.com
- **监控告警**: alerts@smartlottery.com

### 部署工具
- **官方CLI**: `smart-lottery deploy`
- **Web控制台**: [deploy.smartlottery.com](https://deploy.smartlottery.com)
- **API文档**: [api.smartlottery.com/docs](https://api.smartlottery.com/docs)

---

**文档版本**: v1.0.0  
**最后更新**: 2024-08-15  
**维护者**: 智能抽奖系统运维团队