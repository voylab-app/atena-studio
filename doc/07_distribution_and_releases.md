# 🚀 Distribution, Packaging, and Releases Guide — Atena Studio

This guide outlines the compilation, bundling, and distribution workflow for **Atena Studio** across macOS, Windows, and Linux.

---

## 📦 1. Local Release Compilation

To build and package installers locally on your development machine:

```bash
# 1. Install Node/frontend dependencies
pnpm install

# 2. Provision sidecar binaries (llama-server, ffmpeg, uv) — cross-platform
pnpm prepare:binaries

# 3. Compile the application in Release mode
pnpm build
```

Generated bundles will be located under:
* **macOS**: `src-tauri/target/release/bundle/dmg/Atena Studio_0.1.0_aarch64.dmg` (or `app/`)
* **Windows**: `src-tauri/target/release/bundle/msi/` or `nsis/` (`.exe` / `.msi`)
* **Linux**: `src-tauri/target/release/bundle/appimage/` (`.AppImage`) or `deb/` (`.deb`)

---

## 🍎 2. Guidance for macOS Users (Gatekeeper)

When Atena Studio is distributed as a standalone binary or pre-release without an Apple Developer corporate signing certificate:

### Common Symptom
When opening the `.dmg` or `.app` for the first time, macOS may present a warning dialog:
> *"Atena Studio cannot be opened because the developer cannot be verified"* or *"The application is damaged and should be moved to the Trash"*.

### Resolution:
1. **Via Finder**: Right-click (or `Control + Click`) the Atena Studio icon in `/Applications` and select **"Open"**; then confirm in the popup.
2. **Via Terminal (Single fast command)**:
   ```bash
   xattr -cr /Applications/"Atena Studio.app"
   ```
   *(This command clears the quarantine attribute assigned by the browser during download).*

---

## 🤖 3. Automated Publishing via GitHub Actions

The repository includes a configured workflow under `.github/workflows/release.yml`.

### How to release a new version:
1. Bump the version in `package.json` and `src-tauri/tauri.conf.json` (e.g., `0.2.0`).
2. Commit changes and push a git tag prefixed with `v`:
   ```bash
   git tag v0.2.0
   git push origin v0.2.0
   ```
3. GitHub Actions triggers builds across matrix targets:
   * **macOS Apple Silicon (ARM64)** -> `.dmg`
   * **Windows (x64)** -> `.msi` / `.exe` installer
   * **Linux (x64)** -> `.AppImage` and `.deb`
4. Once builds finish, the release is automatically drafted or published in the repository's **Releases** tab with all binaries attached.

---

## 🛠️ 4. Bundled Sidecar Structure

Atena bundles local sidecars to ensure 100% offline functionality without mandatory external toolchains:

| Sidecar | Role | Bundle Location |
| :--- | :--- | :--- |
| `llama-server` | Hardware-accelerated GGUF model execution | `binaries/llama-server-<target>` |
| `ffmpeg` | High-speed audio processing for Whisper transcription | `binaries/ffmpeg-<target>` |
| `uv` | Isolated Python runtime and environment manager | `binaries/uv-<target>` |

The `scripts/setup-sidecars.js` runner automatically delegates to `scripts/setup-sidecars.ps1` (Windows) or `scripts/setup-sidecars.sh` (macOS/Linux) to provision these dependencies.
