# ![](logo.png) Aiter

[![Rust](https://github.com/vvlookman/aiter/actions/workflows/build.yml/badge.svg)](https://github.com/vvlookman/aiter/actions/workflows/build.yml)

[English](README.md) &nbsp; 中文

深度理解内容的轻量级 AI 助手

所谓「深度理解」是指除了对内容文本进行切分外，还进行了分段概括、隐性信息提取、生成知识点等处理，使 AI 助手能够更全面地理解并召回内容。

## 快速入门

### 使用 App

[下载](https://github.com/vvlookman/aiter/releases) App 并运行

> **注意** 因为当前自动编译的安装包没有签名，因此 macOS 用户安装后需要手动执行 `xattr -d com.apple.quarantine -r /Applications/Aiter.app`，否则会提示文件已损坏。

### 通过 Docker 运行 Web 服务

**注意** 通过 Web 服务只能访问部分功能，完整功能可通过 App 或命令行使用。还可以通过 App 中的「远程」功能连接 Web 服务。

```sh
docker run --name aiter -itd -v ~/Library/Application\ Support/aiter:/root/.local/share/aiter -p 6868:6868 vvlookman/aiter
```

### 使用命令行

使用命令行最简便的途径是通过 `docker exec -it aiter bash` 命令进入到 Docker 容器中，然后运行 `aiter` 命令。

下面以默认 AI 助手为例：

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
# 先编译 Web 界面，编译结果会打包到命令行程序中
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
docker build -t aiter:${version} .
```
