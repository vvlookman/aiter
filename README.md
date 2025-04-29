# ![](logo.png) Aiter

English [中文](README.zh-CN.md)

Lightweight AI assistant based on deep content understanding

## Quick Start

### Use App

Download the app and run it.

### Use CLI

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

### Run Web Service

```sh
aiter serve
```

The default service runs at http://localhost:6868, **Notice** Only some features can be accessed via the web, all features need to be used via the App or CLI.

### Run Web Service With Docker

```sh
# The data is stored in /root/.local/share/aiter in the container
docker run -it --rm -v ~/Library/Application\ Support/aiter:/root/.local/share/aiter -p 6868:6868 aiter
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
# Build UI for web service, it will be bundled in CLI
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
docker build --no-cache -t aiter:${version} .
```
