# Gmail Archive Viewer

A modern, fast, and local viewer for Gmail archives (Google Takeout).

---

## 📖 For Users

### What is the Gmail Archive Viewer?
When you export your email data from Google via **Google Takeout**, you typically receive a massive `.mbox` file. These files are often many gigabytes in size and are very difficult or impossible to open with standard email clients.

The **Gmail Archive Viewer** solves this:
- **Speed:** Search through thousands of emails in seconds.
- **Familiarity:** The design is based on the well-known Gmail layout.
- **Privacy:** Everything runs locally on your machine. No data is uploaded to the cloud.
- **Completeness:** View and download attachments directly.
- **Launcher Convenience:** A sleek tray menu controls the server and settings.

### How to use the App (Step-by-Step)

#### 1. Data Export
Use [Google Takeout](https://takeout.google.com/) to download your Gmail data. Select **MBOX** as the format. You will receive one or more `.mbox` files once the export is complete.

#### 2. Prepare Data for the App (Conversion)
The app uses an optimized container format (`.mbxc`) to enable lightning-fast searches. You can convert your MBOX files directly within the app:
- Start the **Gmail Archive Viewer**.
- Go to **Settings** (gear icon) -> **Convert**.
- Select your `.mbox` source file and a destination path for the new `.mbxc` file.
- Click **Start Conversion**. The app will now create the index. Your original MBOX file remains unchanged.

#### 3. Settings & New Archives
You don't need to manually edit text files to tell the app what to open – you can do it conveniently through the interface:
- **New Configuration:** In **Settings**, go to the **New Configuration** tab. There you can specify a name for your new `.toml` settings file, select the corresponding `.mbxc` data source, and define filters (e.g., for spam).
- **Switching:** Under **System**, you can switch between different configuration files at any time. The app reloads the data instantly in the background.

#### 4. Using the App
- Once a valid configuration is loaded, the app displays your emails in the familiar Gmail design.
- You can search instantly, filter by labels, and view or download attachments directly in your browser.
- If you don't have data at startup, the **Open Frontend** button in the system menu (tray) remains active so you can open the settings and set up your first archive.

#### 5. Launcher & Tray Menu
The launcher runs in the background (in your system tray / menu bar):
- **Tray Menu:** Accessible via the app icon at the top (Mac) or bottom-right (Windows/Linux):
    - `Open Frontend`: Opens the viewer in your default browser.
    - `System`: Opens the management window for settings.
    - `Quit`: Terminates the app completely.
- **Management Window (System):**
    - **Port:** Change the network port (e.g., 8092) if it's already in use.
    - **Browser:** Optionally choose a specific browser for viewing.
    - **Live Logs:** An integrated window shows real-time messages from the backend – useful for monitoring the loading process or troubleshooting.

---

## 🛠 For Developers (Build Process)

This guide is for developers who want to compile the app themselves or work on it.

### Prerequisites
Make sure the following tools are installed on your system:
- **Rust & Cargo:** (For backend and tools) -> [Installation Guide](https://rustup.rs/)
- **Node.js & npm:** (For the frontend and the Tauri launcher)
- **Just:** (Optional, but recommended as a command runner)

### Project Structure
- `/frontend`: The Svelte-based web frontend.
- `/backend`: The Rust server that reads emails from the ZIP archive.
- `/launcher`: The Tauri app that bundles backend and frontend into a desktop window.
- `/tools/mbox2zip`: The conversion tool (MBOX -> ZMBOX).

### Build Steps

You can manage everything conveniently via the `justfile` (if `just` is installed):

#### 1. Build Everything (Release Mode)
```bash
just release
```
This builds the frontend, the conversion tool, the backend, and the launcher in sequence.

#### 2. Manual Steps (if `just` is not available)

**A. Build Frontend:**
```bash
cd frontend
npm install
npm run build
```

**B. Build Conversion Tool (`mbox2zip`):**
```bash
cd tools/mbox2zip
cargo build --release
```

**C. Build Launcher (Tauri App):**
```bash
cd launcher
npm install
npm run tauri build
```
The finished binaries can be found in the `builds/` folder (after running `./cpybinaries.sh`) or in the respective `target/release` directories.

### Development (Debug Mode)
To start the app during development:
- Start the launcher in development mode:
  ```bash
  cd launcher
  npm run tauri dev
  ```
- This automatically starts hot-reloading for the frontend and recompiles the backend upon changes.
