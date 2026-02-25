# Gmail Archive Viewer

Ein moderner, schneller und lokaler Viewer für Gmail-Archive (Google Takeout).

---

## 📖 Für Anwender

### Was ist der Gmail Archive Viewer?
Wenn du deine E-Mails von Google über **Google Takeout** exportierst, erhältst du meist eine riesige `.mbox`-Datei. Diese Dateien sind oft viele Gigabyte groß und lassen sich mit normalen E-Mail-Programmen nur sehr mühsam oder gar nicht öffnen.

Der **Gmail Archive Viewer** löst dieses Problem:
- **Geschwindigkeit:** Er durchsucht tausende E-Mails in Sekunden.
- **Vertrautheit:** Das Design ist an das bekannte Gmail-Layout angelehnt.
- **Privatsphäre:** Alles läuft lokal auf deinem Rechner. Keine Daten werden in die Cloud hochgeladen.
- **Vollständigkeit:** Anhänge können direkt angesehen und heruntergeladen werden.

### So nutzt du die App

### So nutzt du die App (Schritt-für-Schritt)

#### 1. Datenexport
Nutze [Google Takeout](https://takeout.google.com/), um deine Gmail-Daten herunterzuladen. Wähle dabei das Format **MBOX**. Du erhältst nach Abschluss des Exports eine oder mehrere `.mbox`-Dateien.

#### 2. Daten für die App vorbereiten (Indizierung)
Die App benötigt ein optimiertes Container-Format (`.mbxc`), um schnell suchen zu können. Dabei wird deine Original-MBOX-Datei ausgelesen und ein Index erstellt. Deine `.mbox`-Datei bleibt dabei unverändert erhalten.

Führe das mitgelieferte Tool `mbox2zip` aus:
```bash
# Beispiel für die Nutzung
./mbox2zip --input meine_mails.mbox --output archiv.mbxc
```
*Hinweis: Dieser Vorgang erzeugt die `.mbxc`-Datei, die alle Mails und den Suchindex in komprimierter Form enthält.*

#### 3. Einstellungen konfigurieren
Die App benötigt Informationen darüber, welche Datei sie öffnen soll.
- Kopiere die Datei `settingsample.toml` und nenne sie `settings.toml`.
- Öffne die `settings.toml` in einem Texteditor deiner Wahl.
- Trage unter `zip_path` den vollständigen Pfad zu deiner im vorigen Schritt erstellten `.mbxc`-Datei ein:
  ```toml
  zip_path = "/Dein/Pfad/zu/archiv.mbxc"
  ```
- Optional: Du kannst unter `filter_labels` Labels definieren, die in der App ausgeblendet werden sollen (z.B. "Spam" oder "Wichtig").

#### 4. App starten & Nutzen
- Starte den **Gmail Archive Viewer**.
- Die App lädt beim Start automatisch die in der `settings.toml` hinterlegte Datei.
- Beim ersten Start wird die `.mbxc`-Datei eingelesen und ein Index erstellt. Dies kann je nach Größe der Datei einige Zeit dauern.
- Mit "Settings" kann die `.mbxc`-Datei gewechselt werden. Das ist beim ersten Start notwendig da die App den Pfad zur `.toml`-Datei nicht automatisch finden kann.
- Du kannst nun in der Weboberfläche der App blitzschnell durch deine E-Mails browsen, nach Absendern oder Betreffzeilen suchen und Anhänge öffnen.

---

## 🛠 Für Entwickler (Build-Prozess)

Diese Anleitung richtet sich an Entwickler, die die App selbst kompilieren oder daran arbeiten möchten.

### Voraussetzungen
Stelle sicher, dass folgende Werkzeuge auf deinem System installiert sind:
- **Rust & Cargo:** (Für Backend und Tools) -> [Installationsanleitung](https://rustup.rs/)
- **Node.js & npm:** (Für das Frontend und den Tauri-Launcher)
- **Just:** (Optional, aber empfohlen als Command-Runner)

### Projektstruktur
- `/frontend`: Das Svelte-basierte Web-Frontend.
- `/backend`: Der Rust-Server, der die E-Mails aus dem ZIP-Archiv liest.
- `/launcher`: Die Tauri-App, die Backend und Frontend in einem Desktop-Fenster bündelt.
- `/tools/mbox2zip`: Das Konvertierungstool (MBOX -> ZMBOX).

### Build-Schritte

Du kannst alles bequem über das `justfile` steuern (falls `just` installiert ist):

#### 1. Alles bauen (Release-Modus)
```bash
just release
```
Dies baut nacheinander das Frontend, das Konvertierungstool, das Backend und den Launcher.

#### 2. Manuelle Schritte (falls kein `just` vorhanden)

**A. Frontend bauen:**
```bash
cd frontend
npm install
npm run build
```

**B. Konvertierungstool (`mbox2zip`) bauen:**
```bash
cd tools/mbox2zip
cargo build --release
```

**C. Launcher (Tauri-App) bauen:**
```bash
cd launcher
npm install
npm run tauri build
```
Die fertigen Binärdateien findest du danach im Ordner `builds/` (nach Ausführung von `./cpybinaries.sh`) oder im jeweiligen `target/release` Verzeichnis.

### Entwicklung (Debug-Modus)
Um die App während der Entwicklung zu starten:
- Starte den Launcher im Entwicklungsmodus:
  ```bash
  cd launcher
  npm run tauri dev
  ```
- Dies startet automatisch den Hot-Reload für das Frontend und kompiliert das Backend neu bei Änderungen.
