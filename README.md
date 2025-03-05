# Haku - Quick Search Bar

Haku is a lightweight desktop search bar application built with Tauri, Rust, Vue, and Tailwind CSS. It provides a quick way to search the web using various search engines.

## Features

- Minimal, clean search interface
- Support for multiple search engines (Google, Bing, DuckDuckGo, Yahoo)
- Global shortcut (Shift+Space) to show/hide the search bar
- Automatically hides when not in use
- System tray icon for easy access

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) or [Bun](https://bun.sh/)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

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
bun run tauri build
```

## Usage

1. Press `Shift+Space` to open the search bar
2. Type your search query
3. Press `Enter` to search
4. Click the settings icon to change the search engine

## License

MIT
