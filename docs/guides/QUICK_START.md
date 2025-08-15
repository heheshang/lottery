# 🚀 快速开始指南

> **文档版本**: v1.0.0  
> **创建日期**: 2024-08-15  
> **最后更新**: 2024-08-15  

## 📋 准备工作

### 系统要求
| 系统 | 最低版本 | 推荐版本 |
|---|---|---|
| **Windows** | Windows 10 | Windows 11 |
| **macOS** | macOS 10.15 | macOS 12+ |
| **Linux** | Ubuntu 18.04 | Ubuntu 22.04+ |

### 开发环境要求
```bash
# 必需工具
Node.js     ≥ 18.0.0
Rust        ≥ 1.75.0 (2024 edition)
pnpm        ≥ 8.0.0
Git         ≥ 2.0.0

# 系统依赖
Windows: Microsoft Visual Studio C++ Build Tools
macOS:   Xcode Command Line Tools
Linux:   build-essential, libwebkit2gtk-4.0-dev
```

## 🎯 5分钟上手

### 步骤1: 克隆项目
```bash
# 克隆仓库
git clone https://github.com/your-org/smart-lottery.git
cd smart-lottery

# 查看项目结构
ls -la
```

### 步骤2: 安装依赖
```bash
# 安装前端依赖
pnpm install

# 安装Rust依赖
cd src-tauri
cargo build --release
cd ..

# 验证安装
pnpm --version    # 应显示 ≥8.0.0
rustc --version   # 应显示 ≥1.75.0
cargo --version   # 应显示最新版本
```

### 步骤3: 启动开发环境
```bash
# 启动开发服务器
pnpm tauri dev

# 或使用快捷命令
pnpm dev
```

首次启动时，Tauri会自动：
- 🏗️ 编译Rust后端
- ⚡ 启动Vite开发服务器
- 🔥 启用热重载
- 🎯 打开应用窗口

### 步骤4: 验证运行
启动成功后，您应该看到：
- 应用窗口标题: "智能抽奖系统"
- 窗口尺寸: 1200x800
- 默认页面: 欢迎页面

## 🔧 开发环境配置

### 推荐IDE设置

#### VS Code (推荐)
```bash
# 安装推荐扩展
code --install-extension tauri-apps.tauri-vscode
code --install-extension rust-lang.rust-analyzer
code --install-extension bradlc.vscode-tailwindcss
code --install-extension esbenp.prettier-vscode
```

#### 创建 `.vscode/settings.json`
```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "editor.formatOnSave": true,
  "tailwindCSS.includeLanguages": {
    "typescript": "javascript",
    "typescriptreact": "javascript"
  }
}
```

### 环境变量配置
```bash
# 创建 .env.local
touch .env.local

# 添加开发环境变量
echo "VITE_API_URL=http://localhost:1420" > .env.local
echo "TAURI_DEV_HOST=localhost" >> .env.local
```

## 🎮 基础使用

### 创建第一个抽奖规则
```typescript
// 示例代码
import { useState } from 'react';

function CreateLottery() {
  const [participants, setParticipants] = useState([]);
  
  const addParticipant = (name) => {
    setParticipants([...participants, { name, weight: 1 }]);
  };
  
  const startLottery = () => {
    // 调用后端API
    invoke('execute_lottery', { participants });
  };
  
  return (
    // UI组件
  );
}
```

### 测试后端API
```bash
# 测试Tauri命令
curl -X POST http://localhost:1420/
  -H "Content-Type: application/json"
  -d '{"participants": ["Alice", "Bob", "Charlie"]}'
```

## 🐛 常见问题

### 问题1: 构建失败
```bash
# 症状: cargo build 失败
# 解决: 清理并重新构建
cargo clean
pnpm clean
pnpm install
cargo build
```

### 问题2: 端口冲突
```bash
# 症状: 端口1420被占用
# 解决: 修改端口
export TAURI_DEV_HOST=localhost
export TAURI_PORT=1421
```

### 问题3: 权限错误
```bash
# macOS
sudo xattr -r -d com.apple.quarantine /Applications/智能抽奖系统.app

# Linux
sudo chmod +x src-tauri/target/release/smart-lottery
```

### 问题4: 依赖缺失
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev build-essential curl wget libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# macOS
xcode-select --install

# Windows
# 安装Visual Studio Build Tools
```

## 📊 验证安装

### 检查开发环境
```bash
# 运行验证脚本
pnpm run verify

# 手动检查
node --version    # v18+
rustc --version   # 1.75+
cargo --version   # 最新
pnpm --version    # 8+
```

### 测试项目
```bash
# 运行测试
pnpm test
cargo test

# 检查代码质量
pnpm lint
cargo clippy
```

## 🚀 下一步

成功启动项目后，建议：

1. **阅读 [架构文档](../ARCHITECTURE.md)**
2. **查看 [开发指南](../development/DEVELOPMENT.md)**
3. **探索 [API文档](../api/README.md)**
4. **尝试 [用户指南](USER_GUIDE.md)**

## 💡 小贴士

### 开发快捷键
- **Ctrl+R**: 重新加载应用
- **Ctrl+Shift+I**: 打开开发者工具
- **Ctrl+Shift+P**: 全局快捷键激活

### 常用命令
```bash
pnpm tauri dev      # 开发模式
pnpm tauri build    # 生产构建
pnpm tauri info     # 查看系统信息
pnpm tauri dev --release # 发布模式调试
```

## 🆘 获取帮助

- **遇到问题**: 查看 [故障排除](TROUBLESHOOTING.md)
- **需要帮助**: [GitHub Issues](https://github.com/your-org/smart-lottery/issues)
- **社区支持**: [GitHub Discussions](https://github.com/your-org/smart-lottery/discussions)

---

**恭喜！** 🎉 现在你已经成功搭建了智能抽奖系统的开发环境，可以开始开发了！