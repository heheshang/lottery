# 智能抽奖系统架构文档

> **文档版本**: v1.0.0  
> **创建日期**: 2024-08-15  
> **最后更新**: 2024-08-15  
> **作者**: AI助手  

## 📋 目录

1. [项目概述](#项目概述)
2. [技术栈](#技术栈)
3. [项目结构](#项目结构)
4. [核心架构](#核心架构)
5. [数据模型](#数据模型)
6. [开发指南](#开发指南)
7. [部署指南](#部署指南)
8. [性能优化](#性能优化)
9. [安全规范](#安全规范)
10. [扩展设计](#扩展设计)
11. [故障排除](#故障排除)
12. [版本历史](#版本历史)

---

## 项目概述

**智能抽奖系统**是一个基于现代技术栈构建的跨平台桌面应用，结合AI技术实现智能化抽奖体验。

### 核心特性
- 🎯 **智能抽奖**: 基于AI算法的智能抽奖机制
- 🔍 **本地存储**: DuckDB嵌入式数据库，支持向量搜索
- ☁️ **云端同步**: 可选的云端数据同步功能
- 🔐 **安全认证**: OAuth2认证与数据加密
- 📊 **统计分析**: 完整的抽奖数据分析
- 🎨 **现代UI**: 响应式设计，支持暗黑模式

### 系统架构
```
┌─────────────────────────────────────────┐
│            用户界面层                     │
│        React + TypeScript               │
│    Shadcn UI + Tailwind CSS            │
└─────────────────┬───────────────────────┘
                  │ Tauri IPC
┌─────────────────┴───────────────────────┐
│           业务逻辑层                     │
│         Tauri Backend                   │
│    Rust + tokio + async                │
├─────────────────┬───────────────────────┤
│  AI服务集成     │    存储抽象            │
│  OpenAI API    │  DuckDB + PostgreSQL   │
└─────────────────┴───────────────────────┘
                  │
┌─────────────────┴───────────────────────┐
│           基础设施层                     │
│   AWS/GCP/Azure 云服务                  │
│   Kinesis + ClickHouse 分析             │
└─────────────────────────────────────────┘
```

---

## 技术栈

### 前端技术栈
| 技术 | 版本 | 用途 | 状态 |
|---|---|---|---|
| **React** | 18.3.1 | UI框架 | ✅ 已集成 |
| **TypeScript** | 5.8.3 | 类型系统 | ✅ 已集成 |
| **Vite** | 6.0.3 | 构建工具 | ✅ 已集成 |
| **Tailwind CSS** | 4.1.5 | 样式框架 | ✅ 已集成 |
| **Shadcn UI** | latest | 组件库 | ✅ 已集成 |
| **Zustand** | 5.0.4 | 状态管理 | ✅ 已集成 |

### 后端技术栈
| 技术 | 版本 | 用途 | 状态 |
|---|---|---|---|
| **Tauri** | 2.x | 桌面框架 | ✅ 已集成 |
| **Rust** | 2024 edition | 后端语言 | ✅ 已集成 |
| **Tokio** | 1.x | 异步运行时 | ✅ 已集成 |
| **DuckDB** | latest | 本地数据库 | 🔄 待集成 |
| **PostgreSQL** | 15+ | 云端数据库 | 🔄 待集成 |

---

## 项目结构

```
smart-lottery/                                    # 项目根目录
├── 📁 src/                                      # 前端源码 (2024-08-15)
│   ├── 📁 components/                           # React组件
│   │   └── 📁 ui/                              # Shadcn UI组件库
│   │       ├── button.tsx                      # 按钮组件 (已集成)
│   │       ├── card.tsx                        # 卡片组件 (已集成)
│   │       ├── input.tsx                       # 输入组件 (已集成)
│   │       └── ...                             # 其他UI组件
│   ├── 📁 stores/                              # Zustand状态管理
│   ├── 📁 lib/                                # 工具函数
│   │   └── utils.ts                           # 工具函数 (已集成)
│   ├── 📁 assets/                             # 静态资源
│   ├── App.tsx                                # 主应用组件
│   └── main.tsx                               # 应用入口
├── 📁 src-tauri/                              # Tauri后端 (2024-08-15)
│   ├── 📁 src/
│   │   ├── commands.rs                       # Tauri命令 (已集成)
│   │   ├── state.rs                          # 应用状态 (已集成)
│   │   ├── main.rs                           # 应用入口 (已集成)
│   │   └── lib.rs                            # 库入口 (已集成)
│   ├── 📁 capabilities/                      # 权限配置
│   │   └── default.json                      # 默认权限配置 (已集成)
│   ├── 📁 icons/                             # 应用图标
│   ├── 📁 gen/schemas/                       # 生成配置
│   ├── tauri.conf.json                       # Tauri配置 (已集成)
│   ├── Cargo.toml                           # Rust配置 (已集成)
│   └── build.rs                             # 构建脚本 (已集成)
├── 📁 app-core/                             # 核心业务库 (2024-08-15)
│   ├── 📁 src/
│   │   ├── lib.rs                           # 库入口 (已集成)
│   │   ├── models.rs                        # 数据模型 (待完善)
│   │   ├── error.rs                         # 错误处理 (已集成)
│   │   └── tests/                           # 单元测试
│   └── Cargo.toml                          # 库配置 (已集成)
├── 📁 specs/                                # 技术文档
├── 📁 public/                               # 静态资源 (2024-08-15)
│   ├── tauri.svg
│   └── vite.svg
├── 📁 docs/                                 # 项目文档
├── 📁 dist/                                 # 构建输出
└── 📝 配置文件
    ├── package.json                        # 前端依赖 (已集成, 2024-08-15)
    ├── pnpm-lock.yaml                      # 依赖锁定 (已集成, 2024-08-15)
    ├── Cargo.lock                          # Rust依赖锁定 (已集成, 2024-08-15)
    ├── Cargo.toml                          # Rust工作空间 (已集成, 2024-08-15)
    ├── tauri.conf.json                     # Tauri配置 (已集成, 2024-08-15)
    ├── deny.toml                           # 安全检查配置 (已集成, 2024-08-15)
    ├── components.json                     # Shadcn配置 (已集成, 2024-08-15)
    ├── tsconfig.json                       # TypeScript配置 (已集成, 2024-08-15)
    ├── tsconfig.app.json                   # 应用TS配置 (已集成, 2024-08-15)
    ├── tsconfig.node.json                  # Node TS配置 (已集成, 2024-08-15)
    ├── vite.config.ts                      # Vite配置 (已集成, 2024-08-15)
    └── README.md                           # 项目说明 (已集成, 2024-08-15)
```

---

## 核心架构

### 前端架构
```typescript
// src/lib/utils.ts - 工具函数 (2024-08-15)
import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}
```

```typescript
// src/components/ui/button.tsx - UI组件 (2024-08-15)
import * as React from "react"
import { Slot } from "@radix-ui/react-slot"
import { cva, type VariantProps } from "class-variance-authority"
import { cn } from "@/lib/utils"

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all disabled:pointer-events-none disabled:opacity-50",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground shadow-xs hover:bg-primary/90",
        destructive: "bg-destructive text-white shadow-xs hover:bg-destructive/90",
        outline: "border bg-background shadow-xs hover:bg-accent hover:text-accent-foreground",
        secondary: "bg-secondary text-secondary-foreground shadow-xs hover:bg-secondary/80",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-9 px-4 py-2",
        sm: "h-8 rounded-md px-3",
        lg: "h-10 rounded-md px-6",
        icon: "size-9",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)
```

### 后端架构
```rust
// src-tauri/src/lib.rs - 应用入口 (2024-08-15)
mod commands;
mod state;

use anyhow::Result;
use tauri::Manager;

pub use state::AppState;

const APP_PATH: &str = "smart-lottery";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {
    let app_path = dirs::data_local_dir().unwrap().join(APP_PATH);
    if !app_path.exists() {
        std::fs::create_dir_all(&app_path)?;
    }
    let state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::Stdout,
                ))
                .target(tauri_plugin_log::Target::new(
                    tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("access.log".to_string()),
                    },
                ))
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            commands::greet,
        ])
        .setup(|app| {
            app.manage(state);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}
```

```rust
// src-tauri/src/state.rs - 应用状态 (2024-08-15)
use tracing::info;

#[derive(Clone, Default)]
pub struct AppState {}

impl AppState {
    pub fn new() -> Self {
        info!("Initializing state");
        Self {}
    }
}
```

```rust
// src-tauri/src/commands.rs - Tauri命令 (2024-08-15)
use tauri::command;

#[command]
pub(crate) fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust.", name)
}
```

---

## 数据模型

### 数据库设计
```sql
-- 抽奖规则表 (2024-08-15)
CREATE TABLE lottery_rules (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    algorithm_type VARCHAR(50) NOT NULL,
    parameters JSONB,
    is_active BOOLEAN DEFAULT TRUE,
    is_private BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    owner_id UUID NOT NULL
);

-- 抽奖历史表 (2024-08-15)
CREATE TABLE lottery_history (
    id UUID PRIMARY KEY,
    rule_id UUID REFERENCES lottery_rules(id),
    participants JSONB NOT NULL,
    winners JSONB NOT NULL,
    draw_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    metadata JSONB,
    vector FLOAT[] -- 用于AI分析
);

-- 参与者表 (2024-08-15)
CREATE TABLE participants (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    weight FLOAT DEFAULT 1.0,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 应用配置表 (2024-08-15)
CREATE TABLE app_settings (
    key VARCHAR(255) PRIMARY KEY,
    value JSONB,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Rust数据模型
```rust
// app-core/src/models.rs - 数据模型定义 (2024-08-15)
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotteryRule {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub algorithm_type: AlgorithmType,
    pub parameters: serde_json::Value,
    pub is_active: bool,
    pub is_private: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub owner_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlgorithmType {
    Random,
    Weighted,
    AiOptimized,
    Sequential,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub weight: f64,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotteryResult {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub winners: Vec<Participant>,
    pub draw_timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: serde_json::Value,
}
```

```rust
// app-core/src/error.rs - 错误处理 (2024-08-15)
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LotteryError {
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("invalid lottery rule: {0}")]
    InvalidRule(String),
    
    #[error("participant not found: {0}")]
    ParticipantNotFound(Uuid),
    
    #[error("lottery execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("ai service error: {0}")]
    AIServiceError(String),
}

pub type Result<T> = std::result::Result<T, LotteryError>;
```

---

## 开发指南

### 环境要求 (2024-08-15)
- **Node.js**: >= 18.0.0
- **Rust**: >= 1.75.0 (2024 edition)
- **pnpm**: >= 8.0.0
- **系统依赖**: 
  - Windows: Microsoft Visual Studio C++ Build Tools
  - macOS: Xcode Command Line Tools
  - Linux: build-essential, libwebkit2gtk-4.0-dev

### 快速开始
```bash
# 1. 克隆项目 (2024-08-15)
git clone https://github.com/your-org/smart-lottery.git
cd smart-lottery

# 2. 安装依赖
pnpm install
cd src-tauri && cargo build

# 3. 启动开发环境
pnpm tauri dev

# 4. 运行测试
pnpm test
cargo test
```

### 开发工作流
```bash
# 开发模式
pnpm dev           # 启动Vite开发服务器
pnpm tauri dev     # 启动Tauri应用

# 构建
pnpm build         # 构建前端
pnpm tauri build   # 构建桌面应用

# 代码质量
pnpm lint          # 前端代码检查
cargo clippy       # Rust代码检查
cargo fmt          # Rust代码格式化

# 安全检查
cargo deny check   # 依赖安全检查
```

---

## 部署指南

### 本地构建 (2024-08-15)
```bash
# 生产构建
pnpm build
pnpm tauri build

# 输出目录结构
src-tauri/target/release/bundle/
├── msi/                    # Windows安装包
├── dmg/                    # macOS安装包
├── deb/                    # Ubuntu/Debian安装包
├── rpm/                    # CentOS/RHEL安装包
└── appimage/               # AppImage通用包
```

### CI/CD配置
```yaml
# .github/workflows/release.yml (2024-08-15)
name: Release
on:
  push:
    tags: ['v*']

jobs:
  release:
    strategy:
      matrix:
        platform: [macos-latest, ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v2
        with:
          version: 8
      - uses: dtolnay/rust-toolchain@stable
      - uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

## 性能优化

### 前端优化 (2024-08-15)
```typescript
// 代码分割示例
const LazyLotteryEngine = React.lazy(() => import('./components/LotteryEngine'));

// 防抖处理
import { debounce } from 'lodash';
const debouncedSearch = useMemo(
  () => debounce(handleSearch, 300),
  [handleSearch]
);

// 虚拟滚动
import { useVirtualizer } from '@tanstack/react-virtual';
```

### 后端优化 (2024-08-15)
```rust
// 连接池配置
use sqlx::postgres::PgPoolOptions;
let pool = PgPoolOptions::new()
    .max_connections(10)
    .min_connections(2)
    .connect_timeout(Duration::from_secs(10))
    .connect(&database_url)
    .await?;

// 缓存策略
use std::sync::Arc;
use tokio::sync::RwLock;
type ResultCache = Arc<RwLock<HashMap<String, LotteryResult>>>;
```

---

## 安全规范

### 安全检查配置 (2024-08-15)
```toml
# deny.toml 安全配置
[advisories]
ignore = [
    "RUSTSEC-2024-0429",  # 已评估的安全警告
    "RUSTSEC-2025-0012"   # 已知但可控的风险
]

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
    "ISC"
]
```

### 数据安全
- **本地加密**: DuckDB数据文件加密存储
- **传输安全**: 全HTTPS通信
- **密钥管理**: 系统密钥链存储API密钥
- **输入验证**: 前端+后端双重验证

---

## 扩展设计

### 插件系统架构 (2024-08-15)
```rust
// 插件接口定义
pub trait LotteryExtension {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn initialize(&mut self, context: &PluginContext) -> Result<()>;
    fn execute(&self, input: &LotteryInput) -> Result<LotteryResult>;
    fn cleanup(&self) -> Result<()>;
}

// 插件管理器
pub struct PluginManager {
    plugins: Vec<Box<dyn LotteryExtension>>,
}

impl PluginManager {
    pub fn load_plugin(&mut self, plugin: Box<dyn LotteryExtension>) -> Result<()>;
    pub fn execute_plugins(&self, input: &LotteryInput) -> Vec<LotteryResult>;
}
```

### 扩展点
- **算法扩展**: 自定义抽奖算法插件
- **数据源扩展**: 支持多种数据输入源
- **输出扩展**: 多种结果输出格式
- **UI扩展**: 主题和布局自定义

---

## 故障排除

### 常见问题 (2024-08-15)

| 问题 | 症状 | 解决方案 |
|---|---|---|
| **构建失败** | 依赖冲突 | `cargo clean && pnpm clean && pnpm install && cargo build` |
| **权限错误** | macOS无法打开 | `sudo xattr -r -d com.apple.quarantine /Applications/智能抽奖系统.app` |
| **数据库连接** | 连接超时 | 检查数据库文件权限和路径 |
| **内存泄漏** | 应用卡顿 | 使用Valgrind或Instruments分析内存使用 |

### 调试工具
```bash
# 查看日志
tail -f ~/Library/Application\ Support/smart-lottery/access.log  # macOS
tail -f ~/.local/share/smart-lottery/access.log                  # Linux
tail -f %APPDATA%\smart-lottery\access.log                      # Windows

# 调试模式
RUST_LOG=debug cargo tauri dev
```

---

## 版本历史

### v1.0.0 (2024-08-15)
- **初始版本发布**
- ✅ 基础项目结构搭建
- ✅ Tauri + React集成
- ✅ Shadcn UI组件库
- ✅ 基础安全配置
- ✅ 开发环境配置
- ✅ 构建脚本配置

### 变更记录
- **2024-08-15**: 创建项目架构文档
- **2024-08-15**: 初始化项目结构
- **2024-08-15**: 配置开发环境
- **2024-08-15**: 集成基础依赖

### 待办事项
- [ ] DuckDB数据库集成
- [ ] AI算法模块开发
- [ ] 云端同步功能
- [ ] 多语言支持
- [ ] 移动端适配
- [ ] 插件系统实现
- [ ] 性能优化
- [ ] 测试覆盖率提升

---

## 联系与支持

- **项目仓库**: [GitHub](https://github.com/your-org/smart-lottery)
- **问题反馈**: [GitHub Issues](https://github.com/your-org/smart-lottery/issues)
- **文档更新**: 每次PR需更新此文档
- **技术支持**: support@smartlottery.com

---

> **📌 注意**: 本文档随项目同步更新，请始终参考最新版本。
> 
> **最后更新**: 2024-08-15  
> **文档状态**: ✅ 已发布  
> **下一版本**: v1.1.0 (计划中)