<script>
    import { onMount, createEventDispatcher } from "svelte";
    import * as api from "../lib/api";

    export let title = "Datei auswählen";
    export let filter = ""; // e.g., "mbxc,mbox"
    export let mode = "file"; // "file", "folder", or "save"
    export let initialFilename = "";

    const dispatch = createEventDispatcher();

    let currentPath = "";
    let entries = [];
    let drives = [];
    let loading = false;
    let error = null;
    let history = [];
    let filename = initialFilename;

    async function loadDrives() {
        try {
            drives = await api.getDrives();
        } catch (e) {
            console.error("Failed to load drives", e);
        }
    }

    async function navigate(path) {
        loading = true;
        error = null;
        try {
            const res = await api.listDir(path, filter);
            entries = res;
            currentPath = path;
            if (history[history.length - 1] !== path) {
                history = [...history, path];
            }
        } catch (e) {
            error = e.message;
        } finally {
            loading = false;
        }
    }

    function goUp() {
        if (!currentPath) return;

        // Windows path handling
        if (currentPath.length <= 3 && currentPath.includes(":")) {
            currentPath = "";
            entries = [];
            return;
        }

        const parts = currentPath.split(/[/\\]/).filter((p) => p !== "");
        if (parts.length === 0) {
            currentPath = "";
            entries = [];
            return;
        }

        parts.pop();
        let parentPath = parts.join("\\");
        if (currentPath.includes(":") && !parentPath.includes(":")) {
            // Root of a drive
            currentPath = "";
            entries = [];
        } else {
            if (currentPath.startsWith("\\\\")) {
                parentPath = "\\\\" + parentPath;
            }
            navigate(parentPath + (parentPath.endsWith(":") ? "\\" : ""));
        }
    }

    function handleEntryClick(entry) {
        if (entry.is_dir) {
            navigate(entry.path);
        } else if (mode === "file") {
            dispatch("select", entry.path);
        } else if (mode === "save") {
            filename = entry.name;
        }
    }

    function handleSelectClick(entry) {
        if (mode === "file") {
            dispatch("select", entry.path);
        } else if (mode === "save") {
            filename = entry.name;
        }
    }

    function selectFolder() {
        if (mode === "folder" && currentPath) {
            dispatch("select", currentPath);
        }
    }

    function handleSave() {
        if (mode === "save" && currentPath && filename) {
            let fullPath = currentPath;
            if (!fullPath.endsWith("\\") && !fullPath.endsWith("/")) {
                fullPath += currentPath.includes("/") ? "/" : "\\";
            }
            dispatch("select", fullPath + filename);
        }
    }

    onMount(async () => {
        await loadDrives();
        // Try to restore from localStorage or default to first drive/home
        const saved = localStorage.getItem("last_picker_path");
        if (saved) {
            navigate(saved);
        } else if (drives.length > 0) {
            navigate(drives[0].path);
        }
    });

    $: if (currentPath) {
        localStorage.setItem("last_picker_path", currentPath);
    }

    function formatSize(bytes) {
        if (!bytes) return "";
        const units = ["B", "KB", "MB", "GB"];
        let size = bytes;
        let unitIndex = 0;
        while (size > 1024 && unitIndex < units.length - 1) {
            size /= 1024;
            unitIndex++;
        }
        return `${size.toFixed(1)} ${units[unitIndex]}`;
    }
</script>

<div
    class="modal-backdrop"
    on:click|self={() => dispatch("close")}
    on:keydown={(e) => e.key === "Escape" && dispatch("close")}
    role="button"
    tabindex="-1"
>
    <div class="picker-window">
        <div class="picker-header">
            <h3>{title}</h3>
            <button class="close-btn" on:click={() => dispatch("close")}
                >&times;</button
            >
        </div>

        <div class="picker-toolbar">
            <button class="tool-btn" on:click={goUp} disabled={!currentPath}>
                <span class="icon">↑</span> Oben
            </button>
            <div class="path-display">
                <input
                    type="text"
                    bind:value={currentPath}
                    on:keydown={(e) =>
                        e.key === "Enter" && navigate(currentPath)}
                />
            </div>
        </div>

        <div class="picker-main">
            <div class="sidebar">
                <h4>Laufwerke</h4>
                <div class="drive-list">
                    {#each drives as drive}
                        <button
                            class="drive-item"
                            class:active={currentPath === drive.path}
                            on:click={() => navigate(drive.path)}
                        >
                            <span class="icon">💾</span>
                            {drive.name}
                        </button>
                    {/each}
                </div>
            </div>

            <div class="content">
                {#if loading}
                    <div class="status-msg">Lädt...</div>
                {:else if error}
                    <div class="status-msg error">{error}</div>
                {:else if currentPath === "" && drives.length > 0}
                    <div class="status-msg">Bitte ein Laufwerk auswählen</div>
                {:else}
                    <table class="entry-table">
                        <thead>
                            <tr>
                                <th>Name</th>
                                <th class="col-size">Größe</th>
                            </tr>
                        </thead>
                        <tbody>
                            {#each entries as entry}
                                <tr
                                    class="entry-row"
                                    on:dblclick={() => handleEntryClick(entry)}
                                >
                                    <td class="name-cell">
                                        <span class="icon"
                                            >{entry.is_dir ? "📁" : "📄"}</span
                                        >
                                        <span class="name-text"
                                            >{entry.name}</span
                                        >
                                    </td>
                                    <td class="size-cell"
                                        >{entry.is_dir
                                            ? "--"
                                            : formatSize(entry.size)}</td
                                    >
                                    <td class="action-cell">
                                        {#if !entry.is_dir && mode === "file"}
                                            <button
                                                class="select-row-btn"
                                                on:click={() =>
                                                    handleSelectClick(entry)}
                                                >Wählen</button
                                            >
                                        {:else if entry.is_dir}
                                            <button
                                                class="open-row-btn"
                                                on:click={() =>
                                                    navigate(entry.path)}
                                                >Öffnen</button
                                            >
                                        {/if}
                                    </td>
                                </tr>
                            {/each}
                        </tbody>
                    </table>
                {/if}
            </div>
        </div>

        <div class="picker-footer">
            <div class="selection-info">
                {#if mode === "folder"}
                    Ausgewählter Ordner: <strong
                        >{currentPath || "Keiner"}</strong
                    >
                {:else if mode === "save"}
                    <div class="save-input-group">
                        <label for="save-filename">Dateiname:</label>
                        <input
                            id="save-filename"
                            type="text"
                            bind:value={filename}
                            placeholder="name.ext"
                        />
                    </div>
                {/if}
            </div>
            <div class="footer-buttons">
                <button
                    class="btn btn-secondary"
                    on:click={() => dispatch("close")}>Abbrechen</button
                >
                {#if mode === "folder"}
                    <button
                        class="btn btn-primary"
                        on:click={selectFolder}
                        disabled={!currentPath}>Ordner wählen</button
                    >
                {:else if mode === "save"}
                    <button
                        class="btn btn-primary"
                        on:click={handleSave}
                        disabled={!currentPath || !filename}>Speichern</button
                    >
                {/if}
            </div>
        </div>
    </div>
</div>

<style>
    .modal-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 2000;
        backdrop-filter: blur(2px);
    }

    .picker-window {
        width: 800px;
        height: 600px;
        background: var(--surface-color);
        border-radius: 12px;
        display: flex;
        flex-direction: column;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
        border: 1px solid var(--border-strong);
        overflow: hidden;
    }

    .picker-header {
        padding: 16px 20px;
        border-bottom: 1px solid var(--border-strong);
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: var(--bg-color);
    }

    .picker-header h3 {
        margin: 0;
        font-size: 1.1rem;
        color: var(--text-primary);
    }

    .close-btn {
        background: transparent;
        border: none;
        font-size: 1.5rem;
        cursor: pointer;
        color: var(--text-secondary);
    }

    .picker-toolbar {
        padding: 12px 20px;
        display: flex;
        gap: 12px;
        background: var(--surface-color);
        border-bottom: 1px solid var(--border-strong);
    }

    .tool-btn {
        padding: 6px 12px;
        background: var(--bg-color);
        border: 1px solid var(--border-strong);
        border-radius: 4px;
        color: var(--text-primary);
        cursor: pointer;
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .tool-btn:disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .path-display {
        flex: 1;
    }

    .path-display input {
        width: 100%;
        padding: 6px 10px;
        border-radius: 4px;
        border: 1px solid var(--border-strong);
        background: var(--bg-color);
        color: var(--text-primary);
        font-family: monospace;
    }

    .picker-main {
        flex: 1;
        display: flex;
        overflow: hidden;
    }

    .sidebar {
        width: 180px;
        border-right: 1px solid var(--border-strong);
        padding: 12px;
        background: var(--bg-color);
        overflow-y: auto;
    }

    .sidebar h4 {
        margin: 0 0 10px 4px;
        font-size: 0.8rem;
        text-transform: uppercase;
        color: var(--text-secondary);
    }

    .drive-list {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .drive-item {
        text-align: left;
        padding: 8px 10px;
        background: transparent;
        border: none;
        border-radius: 6px;
        color: var(--text-primary);
        cursor: pointer;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .drive-item:hover {
        background: var(--hover-color);
    }

    .drive-item.active {
        background: var(--accent-color);
        color: white;
    }

    .content {
        flex: 1;
        overflow-y: auto;
        background: var(--surface-color);
    }

    .entry-table {
        width: 100%;
        border-collapse: collapse;
        font-size: 0.9rem;
    }

    .entry-table th {
        text-align: left;
        padding: 10px 15px;
        background: var(--bg-color);
        border-bottom: 1px solid var(--border-strong);
        color: var(--text-secondary);
        position: sticky;
        top: 0;
    }

    .entry-row {
        border-bottom: 1px solid var(--border-subtle);
        transition: background 0.1s;
    }

    .entry-row:hover {
        background: var(--hover-color);
    }

    .name-cell {
        padding: 8px 15px;
        display: flex;
        align-items: center;
        gap: 10px;
        cursor: pointer;
    }

    .icon {
        font-size: 1.1rem;
    }

    .size-cell {
        padding: 8px 15px;
        color: var(--text-secondary);
        text-align: right;
    }

    .action-cell {
        padding: 8px 15px;
        text-align: right;
    }

    .select-row-btn,
    .open-row-btn {
        padding: 4px 10px;
        font-size: 0.8rem;
        border-radius: 4px;
        border: 1px solid var(--border-strong);
        background: var(--surface-color);
        color: var(--text-primary);
        cursor: pointer;
    }

    .select-row-btn:hover {
        background: var(--accent-color);
        color: white;
        border-color: var(--accent-color);
    }

    .status-msg {
        padding: 40px;
        text-align: center;
        color: var(--text-secondary);
    }

    .status-msg.error {
        color: #ef4444;
    }

    .picker-footer {
        padding: 16px 20px;
        border-top: 1px solid var(--border-strong);
        background: var(--bg-color);
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .btn {
        padding: 8px 16px;
        border-radius: 6px;
        cursor: pointer;
        font-weight: 500;
    }

    .btn-secondary {
        background: transparent;
        border: 1px solid var(--border-strong);
        color: var(--text-primary);
    }

    .btn-primary {
        background: var(--accent-color);
        border: 1px solid var(--accent-color);
        color: white;
    }

    .save-input-group {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .save-input-group label {
        font-size: 0.9rem;
        color: var(--text-secondary);
        white-space: nowrap;
    }

    .save-input-group input {
        padding: 6px 12px;
        border-radius: 4px;
        border: 1px solid var(--border-strong);
        background: var(--bg-color);
        color: var(--text-primary);
        min-width: 250px;
    }
</style>
