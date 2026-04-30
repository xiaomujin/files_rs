# LocalFiles

一个基于 Rust + Salvo 的本地文件管理服务，提供上传、下载、列表查看、重命名和删除功能，并内置一个原生前端页面用于直接在浏览器中管理文件。

## 功能特性

- 文件上传（支持最大 10GB），自动处理重名文件
- 文件列表查看，按最新上传时间排序，返回文件大小与 MIME 类型
- 文件下载
- 文件重命名（带重名冲突保护）
- 文件删除
- 统一 API 响应格式，结构化错误码
- 原生前端界面，支持拖拽上传与多文件选择
- 上传进度显示
- 浅色 / 深色主题切换，跟随系统偏好并记住用户选择
- Toast 通知反馈
- 前端静态资源内嵌到二进制中，单文件部署
- 通过 `config.json5` 配置服务参数

## 技术栈

- Rust 2024 Edition
- [Salvo](https://salvo.rs/) Web 框架（含 CORS、静态文件服务）
- Tokio 异步运行时
- RustEmbed 静态资源嵌入
- JSON5 配置文件
- 原生 HTML / CSS / JavaScript 前端

## 项目结构

```text
src/
  config.rs              # 配置加载与 config.json5 管理
  main.rs                # 服务启动入口
  routes.rs              # 路由装配与中间件配置
  response.rs            # 统一 API 响应与错误码定义
  handlers/
    files.rs             # 文件上传、下载、列表、删除、重命名
    frontend.rs          # 嵌入式前端资源服务
  models/
    files.rs             # 文件相关数据结构
  services/
    storage.rs           # 重名文件处理逻辑
  utils/
    filename.rs          # 文件名校验
static/
  index.html             # 前端页面入口
  css/app.css            # 前端样式（含浅色/深色主题）
  js/app.js              # 前端交互逻辑
.github/workflows/
  rust.yml               # GitHub Actions 多平台构建
```

## 运行要求

- Rust nightly 工具链
- Cargo

本项目使用 Rust 2024 Edition，需要 nightly 工具链：

```bash
rustup default nightly
```

## 本地运行

1. 克隆项目
2. 启动服务

```bash
cargo run
```

默认监听地址：

```text
http://0.0.0.0:3000
```

启动后直接在浏览器访问即可使用。

## 配置

项目通过可执行文件同目录下的 `config.json5` 进行配置。首次启动时会自动生成默认配置文件。

```json5
{
    // 绑定地址
    bind: "0.0.0.0",
    // 服务端口号
    port: 3000,
    // 文件上传存储路径
    storage_path: "./uploads",
}
```

| 字段 | 默认值 | 说明 |
| --- | --- | --- |
| `bind` | `0.0.0.0` | 服务绑定地址 |
| `port` | `3000` | 服务端口 |
| `storage_path` | `./uploads` | 上传文件的存储目录 |

程序启动时会自动创建存储目录。

## API 概览

所有 API 返回统一格式：

```json
{
  "code": 0,
  "message": "ok",
  "data": {}
}
```

错误时 `code` 非零，`data` 为 `null`。

### 获取文件列表

```http
GET /api/files
```

### 上传文件

```http
POST /api/upload
Content-Type: multipart/form-data
```

字段：`file`（最大 10GB）

### 下载文件

```http
GET /api/download/{id}
```

### 删除文件

```http
DELETE /api/files/{id}
```

### 重命名文件

```http
PUT /api/files/{id}
Content-Type: application/json
```

请求体：

```json
{
  "new_name": "example.txt"
}
```

### 错误码

| 范围 | 类别 | 示例 |
| --- | --- | --- |
| 100-199 | 通用错误 | 缺少参数、无效请求体、无效 Content-Type |
| 200-299 | 文件校验 | 文件名为空、包含非法字符、文件过大 |
| 300-399 | 文件操作 | 文件不存在、保存/删除/重命名/生成文件名失败 |

## 前端说明

前端页面通过 `RustEmbed` 嵌入到服务二进制中：

- `/{*path}` 提供前端页面与静态资源
- `/storage/{**path}` 直接访问上传文件目录（支持目录列表）

前端支持：

- 拖拽上传与多文件选择
- 上传进度反馈
- 文件操作（下载、重命名、删除）
- 重命名弹窗校验
- 浅色 / 深色主题切换与本地持久化
- 跟随系统主题偏好
- Toast 通知

## 构建

开发检查：

```bash
cargo check
```

发布构建：

```bash
cargo build --release
```

## CI

项目包含 GitHub Actions 多平台构建：

- Linux: `x86_64-unknown-linux-musl`
- Windows: `x86_64-pc-windows-msvc`
- macOS: `aarch64-apple-darwin`

工作流文件：`.github/workflows/rust.yml`

CI 会在 `master` 分支的 push / pull request 时执行构建，并上传产物。

## 注意事项

- 文件名安全校验，禁止包含 `/`、`\`、`..`
- 上传重名文件时自动追加时间戳（`file_YYYYMMDD_HHMMSS.ext`），仍冲突则加计数器
- 当前服务默认开放 CORS（允许所有来源）
- 单文件上传大小限制为 10GB

## License

MIT
