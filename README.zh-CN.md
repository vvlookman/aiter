# ![](logo.png) Aiter

[![Rust](https://github.com/vvlookman/aiter/actions/workflows/rust.yml/badge.svg)](https://github.com/vvlookman/aiter/actions/workflows/rust.yml)

[English](README.md) 中文

深度理解内容的轻量级 AI 助手

## 快速入门

### 使用 App

直接下载 App 并运行

### 使用命令行

以默认 AI 助手为例：

```sh
# 接入 LLM 服务
aiter llm config --protocol openai -O base_url:https://dashscope.aliyuncs.com/compatible-mode/v1 -O api_key:sk-xxx -O model:qwen-max-latest qwen
aiter llm test <question>

# 读取并深入理解文档
aiter read <path_to_doc_or_dir>
aiter digest

# 与 AI 对话
aiter chat <question>
```

创建并使用新的 AI 助手:

```sh
# 创建新的 AI 助手
aiter ai new <ai_name>

# 执行命令时附加 @<ai_name>
aiter read <path_to_doc_or_dir> @<ai_name>
aiter digest --batch 16 --concurrent 8 @<ai_name>
aiter chat <question> @<ai_name>
```

### 运行 Web 服务

```sh
aiter serve
```

服务默认运行在 http://localhost:6868 ，**注意** 通过 Web 服务只能访问部分功能，完整功能可通过 App 或命令行使用。

### 通过 Docker 运行 Web 服务

```sh
# 容器中数据存储路径为 /root/.local/share/aiter
docker run -it --rm -v ~/Library/Application\ Support/aiter:/root/.local/share/aiter -p 6868:6868 aiter
```

## 开发

```sh
# 开发 App
cd <project_root>/appui
npm install
npm run tauri dev

# 开发命令行
cd <project_root>
cargo run --

# 开发 Web
cd <project_root>/webui
npm install
npm run dev
```

## 发布

### 编译 App

```sh
cd <project_root>/appui
npm install
npm run tauri build
```

### 编译命令行

```sh
# 编译 Web 界面，编译结果会打包到命令行程序中
cd <project_root>/webui
npm install
npm run build

cd <project_root>
cargo build --release
```

### 编译 Docker 镜像

```sh
cd <project_root>
version=$(sed -n 's/version = "\(.*\)"$/\1/p' Cargo.toml)
docker build --no-cache -t aiter:${version} .
```
