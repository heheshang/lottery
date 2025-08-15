# 🔧 开发指南

> **文档版本**: v1.0.0  
> **创建日期**: 2024-08-15  
> **最后更新**: 2024-08-15  

## 📋 目录

1. [开发环境](#开发环境)
2. [代码规范](#代码规范)
3. [开发工作流](#开发工作流)
4. [测试指南](#测试指南)
5. [调试技巧](#调试技巧)
6. [贡献指南](#贡献指南)

---

## 开发环境

### IDE配置推荐

#### VS Code 设置
```json
// .vscode/settings.json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--all-features"],
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll": true,
    "source.organizeImports": true
  },
  "typescript.preferences.importModuleSpecifier": "relative",
  "tailwindCSS.includeLanguages": {
    "typescript": "javascript",
    "typescriptreact": "javascript"
  },
  "files.exclude": {
    "**/node_modules": true,
    "**/target": true,
    "**/.DS_Store": true
  }
}
```

#### 推荐扩展
```json
// .vscode/extensions.json
{
  "recommendations": [
    "tauri-apps.tauri-vscode",
    "rust-lang.rust-analyzer",
    "bradlc.vscode-tailwindcss",
    "esbenp.prettier-vscode",
    "ms-vscode.vscode-typescript-next",
    "ms-vscode.vscode-json"
  ]
}
```

### 环境变量

#### 开发环境
```bash
# .env.development
VITE_API_URL=http://localhost:1420
VITE_ENV=development
TAURI_DEV_HOST=localhost
```

#### 生产环境
```bash
# .env.production
VITE_API_URL=https://api.smartlottery.com
VITE_ENV=production
```

---

## 代码规范

### Rust代码规范

#### 格式化配置
```toml
# rustfmt.toml
edition = "2021"
max_width = 100
use_small_heuristics = "Max"
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

#### Clippy配置
```rust
// src-tauri/src/main.rs
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::module_name_repetitions)]
```

#### 代码模板
```rust
// src-tauri/src/commands.rs - 命令模板
use tauri::command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[command]
pub async fn sample_command(input: String) -> Result<ApiResponse<String>, String> {
    // 实现逻辑
    Ok(ApiResponse {
        success: true,
        data: Some(input),
        error: None,
    })
}
```

### TypeScript代码规范

#### ESLint配置
```javascript
// .eslintrc.js
module.exports = {
  root: true,
  env: {
    browser: true,
    es2020: true,
    node: true,
  },
  extends: [
    'eslint:recommended',
    '@typescript-eslint/recommended',
    'plugin:react/recommended',
    'plugin:react-hooks/recommended',
  ],
  parser: '@typescript-eslint/parser',
  parserOptions: {
    ecmaVersion: 2020,
    sourceType: 'module',
    ecmaFeatures: {
      jsx: true,
    },
  },
  plugins: ['react', '@typescript-eslint'],
  rules: {
    'react/react-in-jsx-scope': 'off',
    '@typescript-eslint/explicit-module-boundary-types': 'off',
    '@typescript-eslint/no-unused-vars': ['error', { argsIgnorePattern: '^_' }],
  },
};
```

#### Prettier配置
```json
// .prettierrc
{
  "semi": true,
  "trailingComma": "es5",
  "singleQuote": true,
  "printWidth": 80,
  "tabWidth": 2,
  "useTabs": false
}
```

#### 组件模板
```typescript
// src/components/ComponentTemplate.tsx
import React from 'react';
import { cn } from '@/lib/utils';

interface ComponentProps {
  className?: string;
  children?: React.ReactNode;
}

export const ComponentTemplate: React.FC<ComponentProps> = ({
  className,
  children,
}) => {
  return (
    <div className={cn('component-class', className)}>
      {children}
    </div>
  );
};
```

---

## 开发工作流

### Git工作流

#### 分支策略
```bash
main                    # 生产分支
develop                 # 开发分支
feature/new-feature     # 功能分支
hotfix/critical-bug     # 热修复分支
release/v1.0.0         # 发布分支
```

#### 提交规范
```bash
# 格式: type(scope): description
git commit -m "feat(api): add lottery execution endpoint"
git commit -m "fix(ui): resolve button alignment issue"
git commit -m "docs(readme): update installation instructions"
```

#### 提交类型
- `feat`: 新功能
- `fix`: Bug修复
- `docs`: 文档更新
- `style`: 代码格式
- `refactor`: 代码重构
- `test`: 测试相关
- `chore`: 构建/工具

### 开发命令

#### 常用命令
```bash
# 开发
pnpm dev                    # 启动开发服务器
pnpm tauri dev             # 启动Tauri应用
pnpm build                 # 构建前端
pnpm tauri build           # 构建桌面应用

# 测试
pnpm test                  # 运行前端测试
cargo test                 # 运行Rust测试
pnpm test:watch           # 监听模式测试

# 代码质量
pnpm lint                  # 前端代码检查
cargo clippy              # Rust代码检查
cargo fmt                 # Rust代码格式化
pnpm format               # 前端代码格式化

# 安全检查
cargo audit               # 安全检查
cargo deny check          # 依赖检查
```

#### 快捷脚本
```json
// package.json scripts
{
  "dev": "vite",
  "build": "tsc && vite build",
  "preview": "vite preview",
  "test": "vitest",
  "test:watch": "vitest --watch",
  "lint": "eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0",
  "format": "prettier --write .",
  "tauri": "tauri"
}
```

---

## 测试指南

### 测试策略

#### 测试金字塔
```
             /\
            /  \
           / E2E \
          /        \
         / 集成测试  \
        /            \
       /  单元测试    \
      /________________\
```

#### 测试目录结构
```
src/
├── __tests__/            # 单元测试
├── __mocks__/            # 测试模拟
├── e2e/                  # 端到端测试
└── fixtures/             # 测试数据
```

### 单元测试

#### React组件测试
```typescript
// src/__tests__/Button.test.tsx
import { render, screen } from '@testing-library/react';
import { Button } from '@/components/ui/button';

describe('Button Component', () => {
  it('renders correctly', () => {
    render(<Button>Click me</Button>);
    expect(screen.getByText('Click me')).toBeInTheDocument();
  });

  it('applies variant classes', () => {
    render(<Button variant="destructive">Delete</Button>);
    const button = screen.getByText('Delete');
    expect(button).toHaveClass('bg-destructive');
  });
});
```

#### Rust单元测试
```rust
// app-core/src/tests/lottery_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_lottery_rule_creation() {
        let rule = LotteryRule::new(
            "Test Rule".to_string(),
            AlgorithmType::Random,
            serde_json::json!({}),
        );
        assert_eq!(rule.name, "Test Rule");
        assert!(matches!(rule.algorithm_type, AlgorithmType::Random));
    }

    #[tokio::test]
    async fn test_lottery_execution() {
        // 异步测试示例
    }
}
```

### 集成测试

#### Tauri命令测试
```rust
// src-tauri/tests/commands.rs
#[cfg(test)]
mod tests {
    use tauri::Manager;
    use tauri::test::{mock_context, noop_assets};

    #[test]
    fn test_greet_command() {
        let app = tauri::Builder::default()
            .invoke_handler(tauri::generate_handler![greet])
            .build(mock_context(noop_assets()))
            .unwrap();

        let result = app.run_until(move |app| {
            let result = app.invoke(
                "greet",
                tauri::InvokePayload::from_json(serde_json::json!({"name": "World"}))
            );
            assert_eq!(result, Ok(serde_json::json!("Hello, World! You\'ve been greeted from Rust.")));
            true
        });
    }
}
```

### 端到端测试

#### Playwright配置
```typescript
// e2e/example.spec.ts
import { test, expect } from '@playwright/test';

test('basic functionality', async ({ page }) => {
  await page.goto('/');
  await expect(page).toHaveTitle(/智能抽奖系统/);
  
  // 测试交互
  await page.click('button:has-text("开始抽奖")');
  await expect(page.locator('.result')).toBeVisible();
});
```

---

## 调试技巧

### 前端调试

#### React DevTools
```bash
# 安装React DevTools
npm install -g react-devtools

# 启动调试工具
react-devtools
```

#### 日志调试
```typescript
// 开发环境日志
if (process.env.NODE_ENV === 'development') {
  console.log('Debug:', data);
}

// 日志级别
console.debug('Debug message');
console.info('Info message');
console.warn('Warning message');
console.error('Error message');
```

### 后端调试

#### Rust日志
```rust
use tracing::{debug, info, warn, error};

debug!("Debug data: {:?}", data);
info!("Processing lottery rule: {}", rule_id);
warn!("Deprecated API usage detected");
error!("Failed to execute lottery: {}", error);
```

#### 性能分析
```bash
# 使用cargo flamegraph
cargo install flamegraph
cargo flamegraph --bin smart-lottery

# 内存分析
cargo install cargo-instruments
cargo instruments --release
```

### 网络调试

#### 抓包工具
- **Wireshark**: 网络层抓包
- **Charles**: HTTP/HTTPS抓包
- **Fiddler**: Windows平台抓包

#### API调试
```bash
# 使用curl测试API
curl -X POST http://localhost:1420/api/lottery \
  -H "Content-Type: application/json" \
  -d '{"participants": ["Alice", "Bob"]}'
```

---

## 贡献指南

### 贡献流程

#### 1. Fork项目
```bash
# Fork仓库到个人账户
git clone https://github.com/your-username/smart-lottery.git
cd smart-lottery
git remote add upstream https://github.com/your-org/smart-lottery.git
```

#### 2. 创建功能分支
```bash
git checkout develop
git pull upstream develop
git checkout -b feature/your-feature-name
```

#### 3. 开发规范
```bash
# 代码检查
pnpm lint
cargo clippy

# 运行测试
pnpm test
cargo test

# 格式化代码
pnpm format
cargo fmt
```

#### 4. 提交规范
```bash
# 提交格式
git commit -m "feat(api): add lottery execution endpoint"
git commit -m "fix(ui): resolve button alignment issue"
git commit -m "docs(readme): update installation instructions"
```

#### 5. 提交PR
```bash
git push origin feature/your-feature-name
# 创建Pull Request
```

### PR模板

#### 标题格式
```
[type]: 简短描述

[可选的详细描述]

- 修复了什么问题
- 添加了什么功能
- 影响的范围
```

#### PR检查清单
- [ ] 代码符合规范
- [ ] 测试通过
- [ ] 文档已更新
- [ ] 无冲突
- [ ] 性能测试通过

---

## 🛠️ 开发工具推荐

### 必备工具
- **VS Code**: 主IDE
- **Rust Analyzer**: Rust语言支持
- **Prettier**: 代码格式化
- **ESLint**: 代码检查
- **GitLens**: Git集成

### 辅助工具
- **DBeaver**: 数据库管理
- **Postman**: API测试
- **Docker**: 容器化开发
- **GitKraken**: Git可视化

### 浏览器扩展
- **React DevTools**: React调试
- **Redux DevTools**: 状态管理调试
- **Web Vitals**: 性能监控

---

## 📞 支持渠道

- **技术问题**: [GitHub Issues](https://github.com/your-org/smart-lottery/issues)
- **功能请求**: [GitHub Discussions](https://github.com/your-org/smart-lottery/discussions)
- **开发支持**: dev-support@smartlottery.com
- **社区**: Discord/Slack频道

---

**文档版本**: v1.0.0  
**最后更新**: 2024-08-15  
**维护者**: 智能抽奖系统开发团队