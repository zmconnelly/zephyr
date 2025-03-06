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
bun dev
```

### Building

To build the application for production:

```bash
bun run tauri build
```

## CI/CD

This project uses GitHub Actions for continuous integration and deployment:

- Automatic builds are triggered on merges to the `main` branch
- Windows installers (MSI and NSIS) are automatically generated
- Releases are published to GitHub Releases with version numbers based on the build number

To view the workflow configuration, see [.github/workflows/release.yml](.github/workflows/release.yml).

## Usage

1. Press `Shift+Space` to open the search bar
2. Type your query
3. Load it with bangs `!`
4. Hit `Enter` to search
