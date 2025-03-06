# Zephyr

**One search to rule them all**

Zephyr is a lightweight desktop search bar baked with with all the power of DuckDuckGo's [bangs](https://duckduckgo.com/bangs).

## Features

- Minimal, clean search interface
- Support for multiple search engines (Google, Bing, DuckDuckGo, Yahoo)
- Global shortcut (default: `Shift+Space`) to launch the search bar
- Configurable search engines, custom bangs, and more

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) or [Bun](https://bun.sh/)
- [Tauri CLI](https://tauri.app/v2/guides/getting-started/prerequisites)

### Setup

1. Clone the repository
2. Install dependencies:

```bash
bun install
```

3. Run the development server:

```bash
bun run tauri dev
```

### Building

To build the application for production:

```bash
bun dev
```

## Usage

1. Press `Shift+Space` to open the search bar
2. Type your query
3. Load it with bangs `!`
4. Hit `Enter` to search
