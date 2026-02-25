<script>
    export let labels = [];
    export let onSearch = undefined;
    export let onClose = undefined;

    let sender = "";
    let subject = "";
    let hasAttachment = false;
    let label = "";
    let dateFrom = "";
    let dateTo = "";

    function handleSearch() {
        if (onSearch) {
            onSearch({
                sender,
                subject,
                has_attachment: hasAttachment,
                label,
                date_from: dateFrom,
                date_to: dateTo,
            });
        }
        if (onClose) {
            onClose();
        }
    }

    function handleClose() {
        if (onClose) {
            onClose();
        }
    }

    function handleOverlayClick(e) {
        if (e.target === e.currentTarget) {
            handleClose();
        }
    }

    function handleKeyDown(e) {
        if (e.key === "Escape") {
            handleClose();
        }
    }

    function customSort(a, b) {
        const aVal = String(a).toLowerCase();
        const bVal = String(b).toLowerCase();
        if (aVal.startsWith("_") && !bVal.startsWith("_")) return -1;
        if (!aVal.startsWith("_") && bVal.startsWith("_")) return 1;
        return aVal.localeCompare(bVal);
    }
    $: sortedLabels = [...labels].sort(customSort);
</script>

<div
    class="popup-overlay"
    onclick={handleOverlayClick}
    onkeydown={handleKeyDown}
    role="presentation"
>
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div class="popup" role="dialog" aria-modal="true">
        <header class="popup-header">
            <h3>Erweiterte Suche</h3>
            <button class="close-btn" onclick={handleClose}>×</button>
        </header>

        <div class="popup-content">
            <div class="form-group">
                <label for="sender">Von</label>
                <input
                    type="text"
                    id="sender"
                    bind:value={sender}
                    placeholder="Absender"
                />
            </div>

            <div class="form-group">
                <label for="subject">Betreff</label>
                <input
                    type="text"
                    id="subject"
                    bind:value={subject}
                    placeholder="Betreff der Nachricht"
                />
            </div>

            <div class="form-row">
                <div class="form-group">
                    <label for="dateFrom">Datum von</label>
                    <input type="date" id="dateFrom" bind:value={dateFrom} />
                </div>

                <div class="form-group">
                    <label for="dateTo">bis</label>
                    <input type="date" id="dateTo" bind:value={dateTo} />
                </div>
            </div>

            <div class="form-group">
                <label for="label">Label</label>
                <div class="select-wrapper">
                    <select id="label" bind:value={label}>
                        <option value="">Alle Labels</option>
                        {#each sortedLabels as l}
                            <option value={l}>{l}</option>
                        {/each}
                    </select>
                </div>
            </div>

            <div class="form-group checkbox">
                <div class="checkbox-container">
                    <input
                        type="checkbox"
                        id="hasAttach"
                        bind:checked={hasAttachment}
                    />
                    <label for="hasAttach">Hat Anhang</label>
                </div>
            </div>
        </div>

        <div class="actions">
            <button class="btn btn-text" onclick={handleClose}>Abbrechen</button
            >
            <button class="btn btn-primary" onclick={handleSearch}
                >Suchen</button
            >
        </div>
    </div>
</div>

<style>
    .popup-overlay {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.4);
        z-index: 1000;
        display: flex;
        justify-content: center;
        align-items: flex-start;
        padding-top: 80px;
        backdrop-filter: blur(2px);
    }

    .popup {
        background: var(--surface-color);
        width: 100%;
        max-width: 560px;
        padding: 0;
        box-shadow:
            0 8px 10px 1px rgba(0, 0, 0, 0.3),
            0 3px 14px 2px rgba(0, 0, 0, 0.2);
        border-radius: 8px;
        overflow: hidden;
        animation: slideIn 0.2s ease-out;
        color: var(--text-color);
    }

    @keyframes slideIn {
        from {
            transform: translateY(-20px);
            opacity: 0;
        }
        to {
            transform: translateY(0);
            opacity: 1;
        }
    }

    .popup-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 16px 24px;
        border-bottom: 1px solid var(--border-color);
    }

    .popup-header h3 {
        margin: 0;
        font-size: 1.125rem;
        font-weight: 400;
        color: var(--text-color);
    }

    .close-btn {
        background: none;
        border: none;
        font-size: 1.5rem;
        color: var(--text-secondary);
        cursor: pointer;
        padding: 4px;
        line-height: 1;
        border-radius: 50%;
        width: 32px;
        height: 32px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .close-btn:hover {
        background-color: var(--hover-color);
        color: var(--text-color);
    }

    .popup-content {
        padding: 24px;
    }

    .form-group {
        margin-bottom: 20px;
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    .form-row {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 20px;
    }

    .form-group label {
        color: var(--text-color);
        font-size: 0.8125rem;
        font-weight: 500;
        margin-left: 2px;
    }

    input[type="text"],
    input[type="date"],
    select {
        width: 100%;
        height: 36px;
        padding: 0 12px;
        background-color: var(--input-bg);
        border: 1px solid var(--border-strong);
        border-radius: 4px;
        font-size: 0.875rem;
        color: var(--text-color);
        transition: all 0.2s;
        box-sizing: border-box;
    }

    input:focus,
    select:focus {
        background-color: var(--surface-color);
        border-color: var(--accent-color);
        outline: none;
        box-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
    }

    .select-wrapper {
        position: relative;
    }

    .checkbox-container {
        display: flex;
        align-items: center;
        gap: 12px;
        cursor: pointer;
        padding: 4px 0;
    }

    .checkbox-container input {
        width: 18px;
        height: 18px;
        cursor: pointer;
        accent-color: var(--accent-color);
    }

    .checkbox-container label {
        margin: 0;
        cursor: pointer;
        font-size: 0.875rem;
        font-weight: 400;
        color: var(--text-secondary);
    }

    .actions {
        display: flex;
        justify-content: flex-end;
        align-items: center;
        gap: 12px;
        padding: 16px 24px;
        background-color: var(--bg-color);
        border-top: 1px solid var(--border-color);
    }

    .btn {
        height: 36px;
        padding: 0 24px;
        border-radius: 4px;
        font-size: 0.875rem;
        font-weight: 500;
        cursor: pointer;
        border: 1px solid transparent;
        transition:
            background-color 0.2s,
            box-shadow 0.2s;
    }

    .btn-text {
        background: transparent;
        color: var(--text-secondary);
    }

    .btn-text:hover {
        background-color: var(--hover-color);
        color: var(--text-color);
    }

    .btn-primary {
        background-color: var(--accent-color);
        color: white;
    }

    .btn-primary:hover {
        filter: brightness(1.1);
        box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    }

    @media (max-width: 600px) {
        .popup {
            width: 90%;
            margin: 20px;
        }
        .form-row {
            grid-template-columns: 1fr;
            gap: 0;
        }
    }
</style>
