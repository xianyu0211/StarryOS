# GitHub MCP Server 使用指南

## 概述

GitHub MCP (Model Context Protocol) Server 是一个强大的工具，用于自动化管理GitHub项目。本指南将展示如何在StarryOS项目中使用GitHub MCP Server。

## 核心功能

### 1. 仓库管理
- **自动化仓库创建**: 自动配置仓库设置、分支保护规则
- **权限管理**: 管理团队成员和访问权限
- **Webhook配置**: 自动设置事件通知和集成

### 2. Issue跟踪
- **智能模板**: 自动应用Bug报告和功能请求模板
- **标签管理**: 自动化标签分配和分类
- **进度跟踪**: 监控Issue状态和解决进度

### 3. Pull Request管理
- **代码审查**: 自动化审查流程和状态检查
- **CI/CD集成**: 与GitHub Actions深度集成
- **自动合并**: 配置自动合并规则

### 4. CI/CD流水线
- **自动化构建**: Rust项目专用构建配置
- **测试执行**: 单元测试、集成测试、性能测试
- **安全扫描**: 代码安全性和漏洞检查

## 配置说明

### 1. 环境设置

```bash
# 设置GitHub Token
export GITHUB_TOKEN=ghp_your_github_token_here

# 验证Token有效性
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/user
```

### 2. MCP配置文件

项目中的 `.github/mcp-config.json` 文件包含完整的MCP Server配置：

```json
{
  "mcp_server": {
    "name": "starryos-github-mcp",
    "version": "1.0.0",
    "capabilities": {
      "repository_management": true,
      "issue_tracking": true,
      "pull_request_management": true,
      "ci_cd_integration": true,
      "release_management": true
    }
  }
}
```

### 3. 自动化工作流

#### 3.1 代码推送工作流
```yaml
# .github/workflows/ci.yml
name: CI/CD Pipeline
on: [push, pull_request]
jobs:
  build-and-test:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Build project
        run: make all
      - name: Run tests
        run: make test
```

#### 3.2 Issue自动化
- 自动分配标签基于Issue类型
- 智能分配负责人
- 进度状态自动更新

#### 3.3 PR自动化
- 自动请求代码审查
- CI状态检查
- 合并前自动测试

## 使用示例

### 1. 创建GitHub仓库

```bash
# 使用MCP Server创建仓库
curl -X POST https://api.github.com/user/repos \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  -d '{
    "name": "StarryOS",
    "description": "Rust-based AIoT OS Kernel",
    "private": false,
    "has_issues": true,
    "has_projects": false,
    "has_wiki": false,
    "auto_init": true
  }'
```

### 2. 配置分支保护

```bash
# 设置main分支保护
curl -X PUT https://api.github.com/repos/username/StarryOS/branches/main/protection \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  -d '{
    "required_status_checks": {
      "strict": true,
      "contexts": ["build", "test"]
    },
    "enforce_admins": false,
    "required_pull_request_reviews": {
      "required_approving_review_count": 1
    },
    "restrictions": null
  }'
```

### 3. 创建Issue

```bash
# 创建Bug报告
curl -X POST https://api.github.com/repos/username/StarryOS/issues \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  -d '{
    "title": "[BUG] Memory allocation issue",
    "body": "## Description\nMemory allocation fails under specific conditions.\n\n## Steps to Reproduce\n1. Run memory stress test\n2. Observe allocation failure\n\n## Expected Behavior\nMemory should be allocated successfully.",
    "labels": ["bug", "memory"]
  }'
```

## 高级功能

### 1. Webhook集成

配置Webhook来自动响应仓库事件：

```json
{
  "webhooks": [
    {
      "name": "ci-trigger",
      "active": true,
      "events": ["push", "pull_request"],
      "config": {
        "url": "https://ci.example.com/webhook",
        "content_type": "json"
      }
    }
  ]
}
```

### 2. 自动化发布

配置自动发布流程：

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags:
      - 'v*'
jobs:
  create-release:
    runs-on: ubuntu-latest
    steps:
      - name: Create Release
        uses: actions/create-release@v1
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 3. 代码质量检查

集成代码质量工具：

```yaml
- name: Clippy Check
  run: cargo clippy --all-targets --all-features -- -D warnings

- name: Format Check
  run: cargo fmt -- --check

- name: Security Audit
  run: cargo audit
```

## 故障排除

### 常见问题

1. **Token权限不足**
   - 确保Token具有足够的权限（repo、workflow等）
   - 检查Token是否已过期

2. **API速率限制**
   - 监控API使用情况：`curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/rate_limit`
   - 实现适当的重试机制

3. **Webhook交付失败**
   - 检查目标URL可访问性
   - 验证签名密钥配置

### 调试技巧

```bash
# 启用详细日志
export GITHUB_DEBUG=true

# 检查API响应
curl -i -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/repos/username/StarryOS

# 监控Webhook交付
curl -H "Authorization: token $GITHUB_TOKEN" https://api.github.com/repos/username/StarryOS/hooks
```

## 最佳实践

### 1. 安全性
- 使用细粒度的访问Token
- 定期轮换Token
- 启用2FA认证

### 2. 性能优化
- 批量处理API请求
- 实现适当的缓存策略
- 监控API速率限制

### 3. 错误处理
- 实现健壮的重试机制
- 记录详细的错误日志
- 设置适当的警报

## 扩展功能

### 1. 自定义集成
- 集成外部CI/CD系统
- 连接项目管理工具（Jira、Trello等）
- 集成监控和告警系统

### 2. 数据分析
- 收集开发指标数据
- 分析代码质量趋势
- 监控团队协作效率

---

*文档版本: v1.0*  
*最后更新: 2025-10-22*