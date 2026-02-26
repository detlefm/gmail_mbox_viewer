<script>
    import { onMount } from "svelte";
    import * as api from "../lib/api";

    export let onReload = () => {};

    // Tabs: 'system', 'convert'
    let activeTab = "system";

    let status = "Wird geladen...";
    let settingsPath = "";
    let zipPath = "";
    let browser = "";
    let dbLoaded = false;

    let mboxFile = "";
    let mbxcOutput = "";

    async function loadStatus() {
        try {
            const info = await api.getSystemInfo();
            settingsPath = info.settings_path || "";
            zipPath = info.zip_path || "";
            browser = info.browser || "";
            dbLoaded = info.db_loaded;
            status = "Bereit";
        } catch (err) {
            status = "Fehler: " + err.message;
        }
    }

    async function handleSelectSettingsFile() {
        try {
            const path = await api.selectSettingsFile();
            if (path) {
                if (
                    confirm(
                        "Möchten Sie zu dieser Konfigurationsdatei wechseln und das Backend neu starten?",
                    )
                ) {
                    status = "Wechsle Konfiguration...";
                    await api.restartWithSettings(path);
                    alert(
                        "Konfiguration wird gewechselt. Bitte laden Sie die Seite in Kürze neu.",
                    );
                    onReload();
                }
            }
        } catch (err) {
            alert("Fehler bei Dateiauswahl: " + err.message);
        }
    }

    async function selectZipFile() {
        const path = await api.selectFile();
        if (path) zipPath = path;
    }

    async function saveSettings() {
        try {
            status = "Speichere...";
            await api.updateSettings(zipPath, browser);
            alert("Einstellungen gespeichert. Das Backend wird neu gestartet.");
            onReload();
        } catch (err) {
            alert("Fehler beim Speichern: " + err.message);
        }
    }

    async function selectMbox() {
        const path = await api.selectFile();
        if (path) mboxFile = path;
    }

    async function selectMbxcOutput() {
        const path = await api.selectSaveFile();
        if (path) mbxcOutput = path;
    }

    async function startConversion() {
        if (!mboxFile || !mbxcOutput) {
            alert("Bitte MBOX-Datei und Zielpfad wählen.");
            return;
        }
        try {
            status = "Konvertierung läuft...";
            await api.convertMbox(mboxFile, mbxcOutput);
            alert("Konvertierung abgeschlossen!");
            status = "Bereit";
        } catch (err) {
            alert("Fehler: " + err.message);
            status = "Fehler bei Konvertierung";
        }
    }

    onMount(async () => {
        await loadStatus();
    });
</script>

<div class="management-layout">
    <aside class="sidebar">
        <div class="sidebar-header">
            <h3>Einstellungen</h3>
        </div>
        <nav>
            <button
                class="nav-item"
                class:active={activeTab === "system"}
                on:click={() => (activeTab = "system")}
            >
                <span class="icon">⚙️</span> System
            </button>
            <button
                class="nav-item"
                class:active={activeTab === "convert"}
                on:click={() => (activeTab = "convert")}
            >
                <span class="icon">📦</span> Konvertieren
            </button>
        </nav>
        <div class="sidebar-footer">
            <div class="status-summary">
                <strong>Status:</strong>
                <span class="status-tag" class:error={status.includes("Fehler")}
                    >{status}</span
                >
            </div>
        </div>
    </aside>

    <main class="content-area">
        {#if activeTab === "system"}
            <section class="tab-content">
                <div class="card">
                    <h2>System-Einstellungen</h2>
                    <p class="hint">
                        Hier können Sie die Konfigurationsdatei wechseln oder
                        Anpassungen an der aktuellen Datei vornehmen.
                    </p>

                    <div class="form-section">
                        <div class="row">
                            <label for="settings-path"
                                >Aktuelle Konfigurations-Datei (.toml):</label
                            >
                            <div class="input-group">
                                <input
                                    id="settings-path"
                                    type="text"
                                    bind:value={settingsPath}
                                    readonly
                                />
                                <button
                                    class="secondary-btn"
                                    on:click={handleSelectSettingsFile}
                                    >Andere Datei wählen...</button
                                >
                            </div>
                        </div>

                        <div class="row">
                            <label for="zip-path">Datenquelle (.mbxc):</label>
                            <div class="input-group">
                                <input
                                    id="zip-path"
                                    type="text"
                                    bind:value={zipPath}
                                    readonly
                                />
                                <button
                                    class="secondary-btn"
                                    on:click={selectZipFile}>Wählen...</button
                                >
                            </div>
                        </div>

                        <div class="row">
                            <label for="browser-input">Browser (Pfad):</label>
                            <input
                                id="browser-input"
                                type="text"
                                bind:value={browser}
                                placeholder="Standard-Browser"
                            />
                        </div>

                        <div class="actions">
                            <button class="primary-btn" on:click={saveSettings}
                                >Aktuelle Einstellungen speichern & Neustart</button
                            >
                        </div>
                    </div>
                </div>

                <div class="card info-card">
                    <h3>Info</h3>
                    <p>
                        Die Konfigurationsdatei speichert Pfade,
                        Port-Einstellungen und Darstellungsoptionen. Änderungen
                        daran erfordern einen Neustart des Hintergrunddienstes.
                    </p>
                </div>
            </section>
        {:else if activeTab === "convert"}
            <section class="tab-content">
                <div class="card">
                    <h2>MBOX zu MBXC konvertieren</h2>
                    <p class="hint">
                        Konvertiert eine Google Mail MBOX-Datei in das
                        platzsparende und schnelle MBXC-Format.
                    </p>

                    <div class="form-section">
                        <div class="row">
                            <label for="mbox-source">Quelle (MBOX-Datei):</label
                            >
                            <div class="input-group">
                                <input
                                    id="mbox-source"
                                    type="text"
                                    bind:value={mboxFile}
                                    readonly
                                />
                                <button
                                    class="secondary-btn"
                                    on:click={selectMbox}>Wählen...</button
                                >
                            </div>
                        </div>

                        <div class="row">
                            <label for="mbxc-target">Ziel (.mbxc Datei):</label>
                            <div class="input-group">
                                <input
                                    id="mbxc-target"
                                    type="text"
                                    bind:value={mbxcOutput}
                                    readonly
                                />
                                <button
                                    class="secondary-btn"
                                    on:click={selectMbxcOutput}
                                    >Speichern unter...</button
                                >
                            </div>
                        </div>

                        <div class="actions">
                            <button
                                class="primary-btn"
                                on:click={startConversion}
                                disabled={status.includes("läuft")}
                            >
                                {status.includes("läuft")
                                    ? "Konvertierung läuft..."
                                    : "Konvertierung starten"}
                            </button>
                        </div>
                    </div>
                </div>
            </section>
        {/if}
    </main>
</div>

<style>
    :root {
        --sidebar-width: 240px;
    }

    .management-layout {
        display: flex;
        height: calc(100vh - 64px); /* Subtract header height */
        background: var(--bg-color);
        overflow: hidden;
    }

    .sidebar {
        width: var(--sidebar-width);
        background: var(--surface-color);
        border-right: 1px solid var(--border-color);
        display: flex;
        flex-direction: column;
    }

    .sidebar-header {
        padding: 1.5rem;
        border-bottom: 1px solid var(--border-color);
    }

    .sidebar-header h3 {
        margin: 0;
        font-size: 1rem;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--text-secondary);
    }

    nav {
        flex: 1;
        padding: 1rem 0;
    }

    .nav-item {
        width: 100%;
        display: flex;
        align-items: center;
        padding: 0.75rem 1.5rem;
        border: none;
        background: transparent;
        color: var(--text-color);
        font-size: 0.95rem;
        cursor: pointer;
        transition: all 0.2s;
        text-align: left;
    }

    .nav-item .icon {
        margin-right: 0.75rem;
        font-size: 1.1rem;
    }

    .nav-item:hover {
        background: var(--bg-color);
    }

    .nav-item.active {
        background: var(--bg-color);
        color: var(--accent-color);
        font-weight: 600;
        border-left: 4px solid var(--accent-color);
        padding-left: calc(1.5rem - 4px);
    }

    .sidebar-footer {
        padding: 1rem;
        border-top: 1px solid var(--border-color);
        background: var(--bg-color);
        font-size: 0.8rem;
    }

    .status-summary {
        display: flex;
        flex-direction: column;
        gap: 0.25rem;
    }

    .status-tag {
        font-weight: bold;
        color: var(--accent-color);
    }

    .status-tag.error {
        color: #ef4444;
    }

    .content-area {
        flex: 1;
        padding: 2rem;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
        align-items: center;
    }

    .tab-content {
        width: 100%;
        max-width: 800px;
        display: flex;
        flex-direction: column;
        gap: 2rem;
    }

    .card {
        background: var(--surface-color);
        border: 1px solid var(--border-color);
        border-radius: 12px;
        padding: 1.5rem;
        box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
    }

    .info-card {
        background: rgba(var(--accent-color-rgb, 66, 133, 244), 0.05);
        border-left: 4px solid var(--accent-color);
    }

    h2 {
        margin-top: 0;
        margin-bottom: 0.5rem;
        font-size: 1.5rem;
    }

    .hint {
        color: var(--text-secondary);
        font-size: 0.95rem;
        margin-bottom: 2rem;
    }

    .form-section {
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .row {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    label {
        font-weight: 600;
        font-size: 0.85rem;
        color: var(--text-secondary);
    }

    .input-group {
        display: flex;
        gap: 0.75rem;
    }

    input {
        flex: 1;
        padding: 0.75rem;
        border: 1px solid var(--border-color);
        border-radius: 8px;
        background: var(--bg-color);
        color: var(--text-color);
        font-size: 0.95rem;
    }

    input:read-only {
        background: var(--surface-color);
        opacity: 0.8;
    }

    .secondary-btn {
        padding: 0.75rem 1rem;
        border-radius: 8px;
        border: 1px solid var(--border-color);
        background: var(--surface-color);
        color: var(--text-color);
        font-weight: 600;
        cursor: pointer;
        white-space: nowrap;
    }

    .secondary-btn:hover {
        background: var(--bg-color);
    }

    .primary-btn {
        padding: 0.85rem 1.5rem;
        border-radius: 8px;
        border: none;
        background: var(--accent-color);
        color: white;
        font-weight: 600;
        cursor: pointer;
        transition: opacity 0.2s;
    }

    .primary-btn:hover {
        opacity: 0.9;
    }

    .primary-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .actions {
        margin-top: 1rem;
    }
</style>
