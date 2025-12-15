# SEO Analyzer (SEOINSKT)

**The easy way to analyze and improve your website's Search Engine Optimization (SEO).**

SEO Analyzer (SEOINSKT) is a desktop application designed to help you understand how search engines see your website. It scans your site and provides actionable recommendations to improve your ranking, visibility, and performance—no coding skills required.

## 📥 Download & Install

You don't need to build this from source! You can download the latest version of the application for Windows (and other supported platforms) directly from our Releases page.

1.  Go to the **[Releases Page](https://github.com/dib59/seo/releases)
2.  Look for the latest version.
3.  Download the **installer file**:
    *   **Windows**: Download the `.msi` or `.exe` file (e.g., `app_x.x.x_x64_en-US.msi`).
4.  Run the installer and follow the on-screen instructions.

## 🚀 What is this application used for?

This tool is useful for **business owners, content creators, and marketers** who want to:

*   **Audit Websites**: Automatically scan any webpage to find broken links, missing tags, and performance issues.
*   **Improve Rankings**: Get specific advice on how to fix problems that might be hurting your Google ranking.
*   **Check Accessibility**: Ensure your site is usable by everyone, which is also good for SEO.
*   **Analyze Competitors**: Run the tool on competitor sites to see what they are doing right (or wrong).

---

## 👨‍💻 For Developers

If you are a developer and want to contribute or modify the source code, see the instructions below.

### Tech Stack

- **Frontend**: [Next.js 16](https://nextjs.org/), [React 19](https://react.dev/), [Tailwind CSS v4](https://tailwindcss.com/)
- **Backend/Desktop**: [Tauri v2](https://v2.tauri.app/) (Rust)
- **UI Components**: [Radix UI](https://www.radix-ui.com/)
- **Data Visualization**: [Recharts](https://recharts.org/)

### Prerequisites

- [Node.js](https://nodejs.org/) (v18 or higher)
- [Rust & Cargo](https://www.rust-lang.org/tools/install) (Required for Tauri)

### Development Setup

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd seo
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Run in Development Mode:**

   *   **Desktop App Mode** (Full Tauri app):
       ```bash
       npm run tauri dev
       ```

### Building from Source

To build the executable files yourself:

```bash
npm run tauri build
```
The binaries will be located in `src-tauri/target/release/bundle`.
