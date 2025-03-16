# Zephyr - One Search to Rule Them All

A featherweight search bar at your fingertips. Ditch your browsers search and go straight to the source.

<p>
  <img src="./assets/images/search_bang_options.png" alt="Zephyr in action" width="75%">
</p>

<table width="75%">
  <tr>
    <td>
      <img src="./assets/images/search_bang_1.png" alt="Zephyr in action" width="100%">
    </td>
    <td>
      <img src="./assets/images/search_results_1.png" alt="Zephyr in action" width="100%">
    </td>
  </tr>
  <tr>
    <td>
      <img src="./assets/images/search_bang_2.png" alt="Zephyr in action" width="100%">
    </td>
    <td>
      <img src="./assets/images/search_results_2.png" alt="Zephyr in action" width="100%">
    </td>
  </tr>
</table>

## Any site, at your fingertips

1. Press `Ctrl + Space` to open the search bar, no need to open your browser
2. Type your query
3. Load it with bangs `!`
4. Hit `Enter` to search

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.85.0+)
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
bun run build
```

## CI/CD

This project uses GitHub Actions:

- Automatic builds are triggered on merges to the `main` branch
- Windows installers (NSIS) are automatically created
- Releases are published to GitHub Releases with versions based on the build number

To view the workflow configuration, see [.github/workflows/release.yml](.github/workflows/release.yml).
