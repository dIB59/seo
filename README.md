# SEO Analyzer (SEOINSKT)

**The powerful, easy-to-use desktop application for comprehensive website SEO analysis.**

SEO Analyzer (SEOINSKT) is designed for business owners, marketers, and developers who need deep insights into their website's Search Engine Optimization. Unlike basic online tools, SEOINSKT runs as a high-performance desktop application, allowing for deep site crawls, interactive visualizations, and AI-powered strategy recommendations.

---

## ✨ Key Features

### 🤖 AI-Powered SEO Insights
- **Custom Personas**: Tailor analysis feedback (e.g., "Professional SEO Consultant" or "Direct Marketing Expert").
- **Actionable Strategy**: Get specific, AI-generated recommendations based on your site's unique data.

### 🕵️ Stealth AI Crawler
Built for high-fidelity discovery without being blocked.
- **Browser Impersonation**: Mimic real browser behavior, bypassing basic bot detection.
- **Adjustable Intensity**: Control crawl speed to suit your server's capacity.

### 📊 Interactive Graph Visualization
Visualize your site's internal architecture like never before.
- **Dynamic Link Mapping**: Powered by **Cosmograph**, explore interconnected pages in a real-time, interactive 2D graph.
- **Health Indicators**: Nodes are color-coded by performance and status codes (e.g., red for broken links).
- **In-Degree Sizing**: Instantly identify your most important pages based on internal link authority.

### 🔍 Comprehensive SEO Audit
Automatic detection of critical issues including:
- **Meta Integrity**: Missing or poorly optimized Title tags and Meta Descriptions.
- **Content Structure**: H1-H3 tag analysis, including missing or multiple H1 tags.
- **Content Quality**: Identification of "Thin Content" (under 300 words).
- **Performance**: Real-time page load time monitoring (flagging pages > 3s).
- **Broken Links**: Automatic detection of 4xx and 5xx errors across internal and external links.
- **Technical SEO**: Robots.txt and Sitemap.xml verification, SSL certificate checks, and Structured Data (JSON-LD) detection.

### 📂 Multi-Format Reporting
Export your findings in the format that works for you:
- **PDF**: Professional, print-ready reports for clients.
- **CSV**: Detailed data for deep-dive analysis in Excel or Google Sheets.
- **Text**: Concise summaries for quick updates.

---

## 🚀 Technical Stack

Built with a modern, high-performance stack for a seamless desktop experience:

- **Core/Backend**: [Rust](https://www.rust-lang.org/) & [Tauri v2](https://v2.tauri.app/) (High performance, memory safety, and native OS integration).
- **Frontend**: [Next.js 16](https://nextjs.org/) & [React 19](https://react.dev/).
- **Styling**: [Tailwind CSS v4](https://tailwindcss.com/) & [Radix UI](https://www.radix-ui.com/).
- **Database**: [SQLite](https://www.sqlite.org/) (Local, serverless data storage).
- **Visualization**: [@cosmograph/react](https://cosmograph.app/).

---

## 📥 Getting Started

### Prerequisites

- **Node.js**: v18 or higher.
- **Rust & Cargo**: Required for building the Tauri application.
- **Gemini API Key**: (Optional) For AI-powered insights.

### System Dependencies

Tauri requires platform-specific native libraries. Follow the official guide for your OS:

- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Linux (Debian/Ubuntu)**:
  ```bash
  sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
    libappindicator3-dev librsvg2-dev patchelf cmake
  ```
- **Windows**: Microsoft Visual Studio C++ Build Tools and WebView2

See the full [Tauri v2 prerequisites guide](https://v2.tauri.app/start/prerequisites/) for details.

### Installation

1. **Clone and Install**:
   ```bash
   git clone https://github.com/dIB59/seo.git
   cd seo
   npm install
   ```

2. **Run in Development**:
   ```bash
   npm run tauri dev
   ```

3. **Build Executables**:
   ```bash
   npm run tauri build
   ```
   Binaries will be located in `src-tauri/target/release/bundle`.

---

## 👨‍💻 Contributing

We welcome contributions! Please feel free to submit Pull Requests or open issues for feature requests and bug reports.

### Project Structure

- **Frontend**: `src/` (Next.js / React)
- **Backend**: `src-tauri/src/` (Rust / Tauri)

### Running Tests

```bash
# Frontend tests
npm test

# Frontend lint
npm run lint

# Backend tests (from repo root)
cd src-tauri && cargo test
```

### Important Caveats

- **`src/bindings.ts` is auto-generated** — never edit it by hand. It is regenerated automatically every time `npm run tauri dev` runs.

- **SQLx offline mode** — CI builds without a database (`SQLX_OFFLINE=true`). If you add or change a `sqlx::query!()` macro, regenerate the cache before pushing:
  ```bash
  cd src-tauri
  DATABASE_URL=sqlite:./dev.db cargo sqlx prepare
  ```
  Then commit the updated files in `src-tauri/.sqlx/`.

### Pull Request Workflow

1. Fork the repo and create a branch from `main`.
2. Make your changes and ensure tests pass (`npm test` and `cd src-tauri && cargo test`).
3. Open a Pull Request against `main`.

