<script>
    import { onMount } from "svelte";
    import * as api from "../lib/api";

    export let onReload = () => {};
    export let onClose = () => {};

    // Tabs: 'system', 'convert'
    let activeTab = "system";

    let status = "Wird geladen...";
    let settingsPath = "";
    let zipPath = "";
    let browser = "";
    let dbLoaded = false;

    let mboxFile = "";
    let mbxcOutput = "";
    let convertStatus = { is_running: false, progress_percent: 0, error: null };
    let pendingSettingsPath = "";
    let pendingZipPath = "";
    let pendingBrowser = "";
    let isLoadingPreview = false;
    let isRestarting = false;

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

    async function checkConvertStatus() {
        try {
            const cs = await api.getConvertStatus();
            convertStatus = cs;
        } catch (e) {
            // Ignore
        }
    }

    async function handleSelectSettingsFile() {
        try {
            const path = await api.selectSettingsFile();
            if (path) {
                pendingSettingsPath = path;
                isLoadingPreview = true;
                status = "Prüfe Konfiguration...";
                try {
                    const preview = await api.inspectSettings(path);
                    if (isLoadingPreview) {
                        pendingZipPath = preview.zip_path;
                        pendingBrowser = preview.browser || "";
                        status = "Vorschau geladen.";
                    }
                } catch (e) {
                    if (isLoadingPreview) status = "Vorschau fehlgeschlagen.";
                } finally {
                    isLoadingPreview = false;
                }
            }
        } catch (err) {
            alert("Fehler bei Dateiauswahl: " + err.message);
        }
    }

    async function acceptSettingsChange() {
        if (isRestarting) return;
        isRestarting = true;
        isLoadingPreview = false; // Prevents the preview from overwriting the status

        try {
            status = "Wechsle Konfiguration...";
            const res = await api.restartWithSettings(pendingSettingsPath);
            if (res.status === "success") {
                status = "Konfiguration erfolgreich gewechselt.";
                pendingSettingsPath = "";
                setTimeout(() => onReload(), 500);
            } else {
                alert("Fehler beim Wechseln der Konfiguration.");
                isRestarting = false;
            }
        } catch (err) {
            alert("Fehler beim Wechseln: " + err.message);
            isRestarting = false;
        }
    }

    function cancelSettingsChange() {
        if (pendingSettingsPath) {
            pendingSettingsPath = "";
            pendingZipPath = "";
            pendingBrowser = "";
            status = "Bereit";
        } else {
            onClose();
        }
    }

    async function saveSettings() {
        try {
            status = "Speichere...";
            const res = await api.updateSettings(zipPath, browser);
            if (res.status === "success") {
                status = "Einstellungen gespeichert.";
                setTimeout(() => onReload(), 500);
            } else {
                alert("Fehler beim Speichern der Einstellungen.");
            }
        } catch (err) {
            alert("Fehler beim Speichern: " + err.message);
        }
    }

    async function selectMbox() {
        const path = await api.selectFile();
        if (path) {
            mboxFile = path;
            // Auto-fill output path by replacing extension
            if (path.toLowerCase().endsWith(".mbox")) {
                mbxcOutput = path.slice(0, -5) + ".mbxc";
            } else {
                mbxcOutput = path + ".mbxc";
            }
        }
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
            await api.convertMbox(mboxFile, mbxcOutput);
        } catch (err) {
            alert("Fehler beim Starten: " + err.message);
        }
    }

    async function abortConversion() {
        try {
            await api.abortConvert();
        } catch (err) {
            alert("Fehler beim Abbrechen: " + err.message);
        }
    }

    onMount(() => {
        loadStatus();
        const interval = setInterval(checkConvertStatus, 1000);
        return () => clearInterval(interval);
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
                                >Konfigurations-Datei (.toml):</label
                            >
                            <div class="input-group">
                                <input
                                    id="settings-path"
                                    type="text"
                                    value={pendingSettingsPath || settingsPath}
                                    readonly
                                />
                                <button
                                    class="secondary-btn"
                                    on:click={handleSelectSettingsFile}
                                    >Datei wählen...</button
                                >
                            </div>
                        </div>

                        <div class="row">
                            <label for="zip-path">Datenquelle (.mbxc):</label>
                            <div class="input-group">
                                <input
                                    id="zip-path"
                                    type="text"
                                    value={pendingSettingsPath
                                        ? pendingZipPath
                                        : zipPath}
                                    readonly
                                />
                            </div>
                        </div>

                        <div class="row">
                            <label for="browser-input">Browser (Pfad):</label>
                            <input
                                id="browser-input"
                                type="text"
                                value={pendingSettingsPath
                                    ? pendingBrowser
                                    : browser}
                                readonly
                                placeholder="Standard-Browser"
                            />
                        </div>

                        <div class="actions">
                            <div class="action-buttons">
                                <button
                                    class="primary-btn accept-btn"
                                    disabled={!pendingSettingsPath ||
                                        pendingSettingsPath === settingsPath}
                                    on:click={acceptSettingsChange}
                                    >Akzeptieren</button
                                >
                                <button
                                    class="secondary-btn cancel-btn"
                                    on:click={cancelSettingsChange}
                                    >Abbrechen</button
                                >
                            </div>
                        </div>
                    </div>
                </div>

                <div class="card info-card">
                    <h3>Info</h3>
                    <p>
                        Die Konfigurationsdatei speichert Pfade,
                        Port-Einstellungen und Darstellungsoptionen. Änderungen
                        daran werden sofort übernommen.
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
                                class="primary-btn accept-btn"
                                on:click={startConversion}
                                disabled={convertStatus.is_running ||
                                    !mboxFile ||
                                    !mbxcOutput}
                            >
                                {convertStatus.is_running
                                    ? "Konvertierung läuft..."
                                    : "Konvertierung starten"}
                            </button>
                        </div>

                        {#if convertStatus.is_running || convertStatus.progress_percent > 0 || convertStatus.error}
                            <div class="convert-progress-area">
                                <div class="progress-details">
                                    <span
                                        >{convertStatus.progress_percent}%
                                        abgeschlossen ({convertStatus.current_message}
                                        Nachrichten)</span
                                    >
                                    {#if convertStatus.is_running}
                                        <button
                                            class="abort-btn"
                                            on:click={abortConversion}
                                            >Abbrechen</button
                                        >
                                    {/if}
                                </div>
                                <div class="progress-track">
                                    <div
                                        class="progress-bar"
                                        style:width={`${convertStatus.progress_percent}%`}
                                        class:error={convertStatus.error}
                                    ></div>
                                </div>
                                {#if convertStatus.error}
                                    <p class="error-msg">
                                        Status: {convertStatus.error}
                                    </p>
                                {:else if !convertStatus.is_running && convertStatus.progress_percent === 100}
                                    <p class="success-msg">
                                        Konvertierung erfolgreich abgeschlossen!
                                    </p>
                                {/if}
                            </div>
                        {/if}
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

    .primary-btn,
    .secondary-btn {
        cursor: pointer;
        pointer-events: auto !important;
    }

    .primary-btn:hover:not(:disabled),
    .secondary-btn:hover:not(:disabled) {
        opacity: 0.9;
        cursor: pointer;
    }

    .primary-btn:disabled,
    .secondary-btn:disabled {
        cursor: not-allowed !important;
        opacity: 0.5;
        pointer-events: none;
    }

    .accept-btn {
        background-color: #10b981;
    }

    .accept-btn:disabled {
        background-color: #6ee7b7;
    }

    .cancel-btn {
        border-color: #ef4444 !important;
        color: #ef4444 !important;
    }

    .action-buttons {
        display: flex;
        gap: 1rem;
    }

    .actions {
        margin-top: 1rem;
    }

    .convert-progress-area {
        margin-top: 2rem;
        padding-top: 2rem;
        border-top: 1px solid var(--border-color);
        display: flex;
        flex-direction: column;
        gap: 1rem;
    }

    .progress-details {
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 0.9rem;
        font-weight: 600;
        color: var(--text-secondary);
    }

    .abort-btn {
        background: transparent;
        border: 1px solid #ef4444;
        color: #ef4444;
        padding: 4px 12px;
        border-radius: 6px;
        cursor: pointer;
        font-size: 0.8rem;
    }

    .abort-btn:hover {
        background: #ef4444;
        color: white;
    }

    .progress-track {
        height: 12px;
        background: var(--bg-color);
        border-radius: 6px;
        overflow: hidden;
    }

    .progress-bar {
        height: 100%;
        background: var(--accent-color);
        transition: width 0.3s ease;
    }

    .progress-bar.error {
        background: #ef4444;
    }

    .error-msg {
        color: #ef4444;
        font-size: 0.9rem;
        font-weight: 600;
        margin: 0;
    }

    .success-msg {
        color: #10b981;
        font-size: 0.9rem;
        font-weight: 600;
        margin: 0;
    }
</style>
