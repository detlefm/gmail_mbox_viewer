#!/bin/bash

# Zielverzeichnis erstellen, falls es nicht existiert
TARGET_DIR="builds"
mkdir -p "$TARGET_DIR"

# Quellverzeichnis
SOURCE_DIR="launcher/src-tauri/target/release/bundle"

# Prüfen, ob das Quellverzeichnis existiert
if [ -d "$SOURCE_DIR" ]; then
    echo "Kopiere Binaries von $SOURCE_DIR nach $TARGET_DIR..."
    
    # -R: Rekursiv
    # -p: Erhalte Attribute (wie Permissions)
    # -v: Verbose (optional, zeigt was kopiert wird)
    # Wir benutzen "." am Ende der Quelle, um den Inhalt des Ordners zu kopieren, 
    # nicht das Verzeichnis selbst (entspricht dem Verhalten von Copy-Item mit Destination).
    cp -Rp "$SOURCE_DIR/." "$TARGET_DIR/"
    
    echo "Fertig."
else
    echo "Fehler: Quellverzeichnis $SOURCE_DIR wurde nicht gefunden."
    echo "Bitte stelle sicher, dass das Projekt vorher gebaut wurde."
    exit 1
fi
