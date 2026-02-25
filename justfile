# eml_viewer justfile

# Windows-spezifische Shell-Einstellung
set windows-shell := ["powershell", "-Command"]

# Standard-Ziel
default: debug

# Build alles im Debug-Modus
debug: build-frontend build-mbox2zip-debug build-backend-debug build-launcher-debug

# Build alles im Release-Modus
release: build-frontend build-mbox2zip-release build-backend-release build-launcher-release

# Hilfsvariable für Befehlsverkettung
and := if os() == "windows" { ";" } else { "&&" }

# Frontend
build-frontend:
    cd frontend {{and}} npm install {{and}} npm run build

# mbox2zip
build-mbox2zip-debug:
    cargo build --manifest-path tools/mbox2zip/Cargo.toml

build-mbox2zip-release:
    cargo build --manifest-path tools/mbox2zip/Cargo.toml --release

# Backend
build-backend-debug:
    cargo build --manifest-path backend/Cargo.toml

build-backend-release:
    cargo build --manifest-path backend/Cargo.toml --release

# Launcher
build-launcher-debug: build-frontend
    cd launcher {{and}} npm install {{and}} npm run tauri build -- --debug

build-launcher-release: build-frontend
    cd launcher {{and}} npm install {{and}} npm run tauri build