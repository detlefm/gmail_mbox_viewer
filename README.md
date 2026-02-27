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
- **Launcher-Komfort:** Ein schlankes Tray-Menü steuert Server und Einstellungen.

### So nutzt du die App

### So nutzt du die App (Schritt-für-Schritt)

#### 1. Datenexport
Nutze [Google Takeout](https://takeout.google.com/), um deine Gmail-Daten herunterzuladen. Wähle dabei das Format **MBOX**. Du erhältst nach Abschluss des Exports eine oder mehrere `.mbox`-Dateien.

#### 2. Daten für die App vorbereiten (Konvertierung)
Die App nutzt ein optimiertes Container-Format (`.mbxc`), um blitzschnelle Suchen zu ermöglichen. Du kannst deine MBOX-Dateien direkt in der App konvertieren:
- Starte den **Gmail Archive Viewer**.
- Gehe zu **Einstellungen** (Zahnrad-Icon) -> **Konvertieren**.
- Wähle deine `.mbox`-Quelldatei und einen Zielpfad für die neue `.mbxc`-Datei aus.
- Klicke auf **Konvertierung starten**. Die App erstellt nun den Index. Deine Original-MBOX bleibt dabei unverändert.

#### 3. Einstellungen & neue Archive
Du musst der App nicht manuell über Textdateien sagen, was sie öffnen soll – das geht bequem über die Oberfläche:
- **Neue Konfiguration:** Gehe in den **Einstellungen** auf den Reiter **Neue Konfiguration**. Dort kannst du einen Namen für deine neue `.toml`-Einstellungsdatei vergeben, die passende `.mbxc`-Datenquelle auswählen und Filter (z.B. für Spam) definieren.
- **Wechseln:** Unter **System** kannst du jederzeit zwischen verschiedenen Konfigurationsdateien hin- und herwechseln. Die App lädt die Daten sofort im Hintergrund neu.

#### 4. App nutzen
- Sobald eine gültige Konfiguration geladen ist, zeigt die App deine E-Mails im vertrauten Gmail-Design.
- Du kannst blitzschnell suchen, nach Labels filtern und Anhänge direkt im Browser ansehen oder herunterladen.
- Falls du beim Start noch keine Daten hast, bleibt der Button **Open Frontend** im System-Menü (Tray) dennoch aktiv, damit du die Einstellungen öffnen und dein erstes Archiv einrichten kannst.

#### 5. Launcher & Tray-Menü
Der Launcher läuft im Hintergrund (im System-Tray / Menüleiste):
- **Tray-Menü:** Erreichbar über das App-Icon oben (Mac) oder unten rechts (Windows/Linux):
    - `Open Frontend`: Öffnet den Viewer im Standardbrowser.
    - `System`: Öffnet das Management-Fenster für Einstellungen.
    - `Quit`: Beendet die App vollständig.
- **Management-Fenster (System):**
    - **Port:** Ändere den Netzwerk-Port (z. B. 8000), falls dieser belegt ist.
    - **Browser:** Wähle optional einen spezifischen Browser für die Darstellung.
    - **Live-Logs:** Ein integriertes Fenster zeigt Echtzeit-Nachrichten des Backends an – hilfreich zur Kontrolle des Ladevorgangs oder bei Fehlern.

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
