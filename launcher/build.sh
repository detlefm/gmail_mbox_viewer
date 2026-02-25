# Build main frontend first
cd ../frontend
npm install
npm run build
cd ../launcher

# Build launcher
npm install
npm run tauri build
