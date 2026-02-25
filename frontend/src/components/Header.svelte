<script>
    export let searchQuery = "";
    export let onSearch = undefined;
    export let onOpenAdvanced = undefined;
    export let onToggleTheme = undefined;
    export let theme = "auto";
    export let archiveName = "Keine Datenquelle";

    function handleSearch() {
        if (onSearch) {
            onSearch({ any: searchQuery });
        }
    }

    function handleKeyDown(e) {
        if (e.key === "Enter") handleSearch();
    }

    function openAdvanced() {
        if (onOpenAdvanced) {
            onOpenAdvanced();
        }
    }

    function toggleTheme() {
        if (onToggleTheme) {
            onToggleTheme();
        }
    }
</script>

<header>
    <div class="logo-area">
        <span class="logo-icon">✉️</span>
        <span class="logo-text">GMail Archive</span>
    </div>

    <div class="header-center">
        <div class="search-bar">
            <button class="icon-btn search-icon" on:click={handleSearch}
                >🔍</button
            >
            <input
                type="text"
                placeholder="Search mail"
                bind:value={searchQuery}
                on:keydown={handleKeyDown}
            />
            <button class="icon-btn options-icon" on:click={openAdvanced}>
                ⚙️
            </button>
        </div>
        <div
            class="archive-info"
            class:no-data={archiveName === "Keine Datenquelle"}
        >
            {archiveName}
        </div>
    </div>

    <div class="user-area">
        <button
            class="icon-btn theme-toggle"
            on:click={toggleTheme}
            title="Toggle Dark Mode"
        >
            {theme === "light" ? "🌙" : theme === "dark" ? "☀️" : "🌓"}
        </button>
        <slot />
    </div>
</header>

<style>
    header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 8px;
        background: var(--surface-color);
        border-bottom: 1px solid var(--border-color);
        height: 64px;
        box-sizing: border-box;
    }

    .logo-area {
        display: flex;
        align-items: center;
        width: 250px;
        padding-left: 1rem;
    }

    .logo-icon {
        font-size: 1.5rem;
        margin-right: 0.5rem;
    }

    .logo-text {
        font-size: 1.25rem;
        color: var(--text-secondary);
        font-family: "Product Sans", Arial, sans-serif;
    }

    .header-center {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 20px;
        margin-left: 20px;
    }

    .search-bar {
        width: 100%;
        max-width: 600px;
        background: var(--input-bg);
        border-radius: 8px;
        display: flex;
        align-items: center;
        padding: 0 10px;
        height: 48px;
        transition:
            background 0.1s,
            box-shadow 0.2s;
    }

    .archive-info {
        font-size: 0.9rem;
        color: var(--text-secondary);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        max-width: 300px;
        font-weight: 500;
        opacity: 0.8;
    }

    .archive-info.no-data {
        color: var(--error-color, #ff3b30);
        background: rgba(255, 59, 48, 0.1);
    }

    .search-bar:focus-within {
        background: var(--surface-color);
        box-shadow:
            0 1px 2px 0 rgba(0, 0, 0, 0.1),
            0 1px 3px 1px rgba(0, 0, 0, 0.05);
        border: 1px solid var(--border-strong);
    }

    input {
        flex: 1;
        border: none;
        background: transparent;
        padding: 0 10px;
        font-size: 1rem;
        outline: none;
        height: 100%;
        color: var(--text-color);
    }

    .icon-btn {
        background: none;
        border: none;
        cursor: pointer;
        font-size: 1.2rem;
        color: var(--text-secondary);
        padding: 8px;
        border-radius: 50%;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: background 0.2s;
    }

    .icon-btn:hover {
        background: var(--hover-color);
    }

    .user-area {
        display: flex;
        align-items: center;
        gap: 8px;
        padding-right: 1rem;
        margin-left: auto;
    }

    .theme-toggle {
        margin-right: 0;
    }
</style>
