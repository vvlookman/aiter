# ![](logo.png) Aiter

[![Rust](https://github.com/vvlookman/aiter/actions/workflows/build.yml/badge.svg)](https://github.com/vvlookman/aiter/actions/workflows/build.yml)

English &nbsp; [中文](README.zh-CN.md)

Lightweight AI assistant based on deep content understanding

The term "deep understanding" refers to the process that in addition to content chunking, the AI assistant also performs tasks such as summarizing sections, extracting implicit information, and generating knowledge points. This enables the AI assistant to have a more comprehensive understanding and recall of the content.

## Quick Start

### Use App

[Download](https://github.com/vvlookman/aiter/releases) the app and run it.

> **Notice** Since the current auto-compiled installer is not signed, macOS users will need to manually execute `xattr -d com.apple.quarantine -r /Applications/Aiter.app` after installing it, or else they will be prompted that the file is damaged.

### Run Web Service With Docker

**Notice** Only some features can be accessed via the web, all features need to be used via the App or CLI. You can also connect to Web services through the "Remote" function in the App.

```sh
docker run --name aiter -itd -v ~/Library/Application\ Support/aiter:/root/.local/share/aiter -p 6868:6868 vvlookman/aiter
```

### Use CLI

The easiest way to use the command line is to get into the Docker container with the `docker exec -it aiter bash` command and then run the `aiter` command.

Start with the default AI assistant:

```sh
# Accessing an LLM service
aiter llm config --protocol openai -O base_url:https://dashscope.aliyuncs.com/compatible-mode/v1 -O api_key:sk-xxx -O model:qwen-max-latest qwen
aiter llm test <question>

# Read & digest documents
aiter read <path_to_doc_or_dir>
aiter digest

# Chat with AI
aiter chat <question>
```

Create and use a new AI assistant:

```sh
# Create a new AI assistant
aiter ai new <ai_name>

# Excute commands with @<ai_name>
aiter read <path_to_doc_or_dir> @<ai_name>
aiter digest --batch 16 --concurrent 8 @<ai_name>
aiter chat <question> @<ai_name>
```

## Develop

```sh
# For App
cd <project_root>/appui
npm install
npm run tauri dev

# For CLI
cd <project_root>
cargo run --

# For Web
cd <project_root>/webui
npm install
npm run dev
```

## Release

### Build App

```sh
cd <project_root>/appui
npm install
npm run tauri build
```

### Build CLI

```sh
# Build UI for web service first, it will be bundled in CLI
cd <project_root>/webui
npm install
npm run build

cd <project_root>
cargo build --release
```

### Build Docker Image

```sh
cd <project_root>
version=$(sed -n 's/version = "\(.*\)"$/\1/p' Cargo.toml)
docker build -t aiter:${version} .
```
