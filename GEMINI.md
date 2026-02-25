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
- **API-Endpunkte:**
    - `GET /labels`: Liefert die Liste aller GMail-Labels aus den Metadaten.
    - `POST /query`: Nimmt Suchparameter entgegen (Sender, DateRange, Attachment-Flag, Subject-String).
    - `GET /messages/{id}`: Extrahiert und liefert die EML-Inhalte.
    - `GET /messages/{id}/attachment/{filename}`: Streamt Anhänge direkt aus der EML/ZIP.
    - `GET /system/info`: Liefert die aktuellen Einstellungen.
    

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

