# Bspterm

远程连接管理器 - 基于 Zed 构建的 SSH/Telnet 会话管理工具。

## 功能特性

- SSH/Telnet 会话管理
- 终端自动化规则引擎
- Python 脚本自动化
- 多会话标签页管理
- 快速连接面板

## 安装

### 从 GitHub Release 下载

从 [Releases](https://github.com/tu10ng/wirsterm/releases) 下载最新版本：

| 文件 | 说明 |
|------|------|
| `bspterm-linux-x86_64.tar.gz` | Linux 二进制文件 |
| `bspterm-windows-x86_64.zip` | Windows 二进制文件 |
| `bspterm-config.zip` | 默认配置文件和示例脚本 |

**bspterm-config.zip 包含：**
- `settings/default_terminal_rules.json` - 终端自动化规则（自动登录等）
- `settings/default_highlight_rules.json` - 语义高亮规则（错误/警告/IP/URL 等）
- `scripts/bspterm.py` - Python 脚本客户端库
- `scripts/API.md` - Python 脚本 API 文档
- `scripts/device_online_notify.py` - 示例：设备上线通知脚本
- `scripts/ne5000e_mpu_collector.py` - 示例：NE5000E MPU 数据采集脚本

配置文件放置位置：`~/.config/bspterm/`

### Linux 本地安装
```sh
./script/install-linux
```

### 从源码构建
```sh
cargo run
```

## 开发

参见 [CLAUDE.md](./CLAUDE.md) 获取开发指南。

## 致谢

Bspterm fork 自 [Zed](https://github.com/zed-industries/zed)，一个高性能代码编辑器。
Zed 的编辑器功能在 Bspterm 中作为辅助工具保留。

## 许可证

GPL-3.0-or-later
