# LocalFiles

一个基于 Rust + Salvo 的本地文件管理服务，提供上传、下载、列表查看、重命名和删除功能，并内置一个原生前端页面用于直接在浏览器中管理文件。

## 功能特性

- 文件上传，自动处理重名文件
- 文件列表查看，按最新上传时间排序
- 文件下载
- 文件重命名
- 文件删除
- 原生前端界面，支持拖拽上传
- 上传进度显示
- 浅色 / 深色主题切换，并记住用户选择
- 前端静态资源内嵌到二进制中，部署简单

## 技术栈

- Rust 2024
- [Salvo](https://salvo.rs/) Web 框架
- Tokio 异步运行时
- RustEmbed 静态资源嵌入
- 原生 HTML / CSS / JavaScript 前端

## 项目结构

```text
src/
  config.rs              # 环境配置与存储目录初始化
  main.rs                # 服务启动入口
  routes.rs              # 路由装配
  handlers/
    files.rs             # 文件上传、下载、列表、删除、重命名
    frontend.rs          # 首页和嵌入式前端资源服务
  models/
    files.rs             # 文件相关数据结构
  services/
    storage.rs           # 重名文件处理逻辑
  utils/
    filename.rs          # 文件名校验
static/
  index.html             # 前端页面入口
  css/app.css            # 前端样式
  js/app.js              # 前端交互逻辑
.github/workflows/
  rust.yml               # GitHub Actions 构建流程
```

## 运行要求

- Rust 工具链
- Cargo

当前 CI 使用的是 `nightly` 工具链进行构建，因此本地也建议使用 nightly：

```bash
rustup default nightly
```

## 本地运行

1. 克隆项目
2. 安装依赖并启动服务

```bash
cargo run
```

默认监听地址：

```text
http://127.0.0.1:3000
```

启动后直接在浏览器访问首页即可使用。

## 配置项

项目当前支持通过环境变量配置文件存储目录。

| 环境变量 | 默认值 | 说明 |
| --- | --- | --- |
| `STORAGE_PATH` | `./uploads` | 上传文件的存储目录 |

示例：

```bash
STORAGE_PATH=./data cargo run
```

程序启动时会自动创建存储目录。

## API 概览

### 获取文件列表

```http
GET /api/files
```

### 上传文件

```http
POST /api/upload
Content-Type: multipart/form-data
```

字段：

- `file`: 上传文件

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

请求体示例：

```json
{
  "new_name": "example.txt"
}
```

## 前端说明

前端页面通过 `RustEmbed` 嵌入到服务二进制中：

- `/` 返回前端首页
- `/assets/{**path}` 提供前端静态资源
- `/static/{**path}` 对外暴露上传文件目录

前端支持：

- 拖拽上传
- 上传进度反馈
- 文件操作按钮
- 重命名弹窗校验
- 浅色 / 深色主题切换与本地持久化

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

项目包含 GitHub Actions 工作流：

- Linux: `x86_64-unknown-linux-musl`
- Windows: `x86_64-pc-windows-msvc`

工作流文件：

```text
.github/workflows/rust.yml
```

CI 会在 `master` 分支的 push / pull request 时执行构建，并上传产物。

## 注意事项

- 文件名会做基础安全校验，禁止包含 `/`、`\`、`..`
- 上传重名文件时会自动追加时间戳，避免覆盖已有文件
- 下载接口直接按文件名作为标识符
- 当前服务默认开放 CORS

## License


