# Projekt-Spezifikation: Static EML Archive Viewer (Single User)

## 1. Architektur-Übersicht
- **Typ:** Client/Server (lokal betrieben).
- **Backend:** Rust
- **Frontend:** Svelte (Responsive Design für Desktop & Mobile).
- **Datenquelle:** Eine statische ZIP-Datei, die `.eml`-Dateien und eine `metadata.json`
`metadata.db` enthält.

## 2. Backend-Logik (Rust)
- **Start-Prozess:** - Liest `metadata.db` einmalig komplett in den Speicher.
    - Bereitet die Daten für effiziente Filterung vor (In-Memory SQLite).
- **ZIP-Handling:** - Die ZIP wird nicht entpackt. Einzelne `.eml`-Dateien werden on-demand extrahiert.
- **API-Endpunkte (Präfix `/api`):**
    - **Daten & Suche:**
        - `GET /labels`: Liste aller verfügbaren GMail-Labels.
        - `POST /query`: Suche mit Filtern (Subject, Sender, Date, Attachments, Label).
    - **Nachrichten-Details:**
        - `GET /messages/{id}`: EML-Inhalt und Metadaten einer Mail.
        - `GET /messages/{id}/attachment/{filename}`: Binär-Stream eines Anhangs.
    - **System & Management:**
        - `GET /system/info`: Aktueller Status (Pfade, Port, Ladezustand).
        - `POST /system/settings`: Speichert Änderungen (Browser, ZIP-Pfad) in die aktuelle TOML.
        - `POST /system/restart`: Schaltet auf eine andere `.toml` Konfigurationsdatei um.
        - `POST /system/create-settings`: Erstellt eine neue Konfigurationsdatei und lädt diese.
        - `POST /system/inspect-toml`: Prüft den Inhalt einer `.toml` Datei vor dem Laden.
    - **Konvertierung:**
        - `POST /system/convert`: Startet die MBOX -> MBXC Konvertierung.
        - `GET /system/convert/status`: Status & Fortschritt der laufenden Konvertierung.
        - `POST /system/convert/abort`: Bricht die laufende Konvertierung ab.
    - **Dateiauswahl (via native Dialoge):**
        - `POST /system/select-file` / `/select-save-file`: Dateiauswahl für Archive.
        - `POST /system/select-toml` / `/select-toml-save`: Dateiauswahl für Konfigurationen.
    

## 3. Frontend-Logik (Svelte)
- **UI-Design:** Authentisches GMail-Clone-Layout.
    - **Sidebar:** Dynamische Label-Liste.
    - **Header:** Suchzeile mit Button für ein **Search-Popup** (Erweiterte Suche).
    - **Main:** Nachrichtenliste mit **adaptiver Pagination** (Desktop ~50, Mobile ~15 Einträge).
- **EML-Rendering:** - Anzeige von HTML-Inhalten in einem sicheren IFrame oder bereinigten Container.
    - Anzeige von Metadaten (Von, An, Datum, Betreff, Labels).
    - Download-Schnittstelle für Attachments.

## 4. Technische Anforderungen & Constraints
- **Modus:** Single User, Read-Only.
- **Performance:** Schnelle Filterung der In-Memory Metadaten; direkter Zugriff auf ZIP-Member für Mail-Details.
- **Responsiveness:** Layout muss sich an verschiedene Bildschirmgrößen anpassen.

