<script>
    export let currentMode = "vertical-split"; // 'no-split', 'vertical-split', 'horizontal-split'
    export let onChange = undefined;

    let isOpen = false;

    function setMode(mode) {
        if (onChange) {
            onChange({ mode });
        }
        isOpen = false;
    }

    function toggleDropdown() {
        isOpen = !isOpen;
    }

    function handleKeyDown(e) {
        if (e.key === "Escape") {
            isOpen = false;
        }
    }

    // Close dropdown when clicking outside
    function handleClickOutside(event) {
        if (isOpen && !event.target.closest(".layout-selector")) {
            isOpen = false;
        }
    }
</script>

<svelte:window onclick={handleClickOutside} onkeydown={handleKeyDown} />

<div class="layout-selector">
    <button
        class="layout-btn"
        onclick={(e) => {
            e.stopPropagation();
            toggleDropdown();
        }}
        aria-label="Layout options"
        aria-expanded={isOpen}
    >
        <span class="layout-icon">▦</span>
        <span class="layout-arrow">▼</span>
    </button>

    {#if isOpen}
        <div
            class="dropdown"
            role="menu"
            tabindex="-1"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") e.stopPropagation();
            }}
        >
            <button
                class="dropdown-item"
                class:active={currentMode === "no-split"}
                onclick={() => setMode("no-split")}
            >
                {#if currentMode === "no-split"}
                    <span class="check">◉</span>
                {:else}
                    <span class="space">○</span>
                {/if}
                List Only
            </button>
            <button
                class="dropdown-item"
                class:active={currentMode === "vertical-split"}
                onclick={() => setMode("vertical-split")}
            >
                {#if currentMode === "vertical-split"}
                    <span class="check">◉</span>
                {:else}
                    <span class="space">○</span>
                {/if}
                Vertical Split
            </button>
            <button
                class="dropdown-item"
                class:active={currentMode === "horizontal-split"}
                onclick={() => setMode("horizontal-split")}
            >
                {#if currentMode === "horizontal-split"}
                    <span class="check">◉</span>
                {:else}
                    <span class="space">○</span>
                {/if}
                Horizontal Split
            </button>
        </div>
    {/if}
</div>

<style>
    .layout-selector {
        position: relative;
        display: inline-block;
    }

    .layout-btn {
        background: transparent;
        border: none;
        cursor: pointer;
        padding: 8px 12px;
        display: flex;
        align-items: center;
        gap: 4px;
        color: #5f6368;
        border-radius: 4px;
        font-size: 1rem;
    }

    .layout-btn:hover {
        background: rgba(0, 0, 0, 0.1);
    }

    .layout-icon {
        font-size: 1.1rem;
    }

    .layout-arrow {
        font-size: 0.6rem;
        margin-left: 2px;
    }

    .dropdown {
        position: absolute;
        top: 100%;
        right: 0;
        background: white;
        border-radius: 8px;
        box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
        min-width: 180px;
        z-index: 1000;
        padding: 8px 0;
        margin-top: 4px;
    }

    .dropdown-item {
        display: flex;
        align-items: center;
        gap: 12px;
        width: 100%;
        padding: 10px 16px;
        background: transparent;
        border: none;
        text-align: left;
        cursor: pointer;
        font-size: 0.9rem;
        color: #202124;
        transition: background 0.1s;
    }

    .dropdown-item:hover {
        background: #f1f3f4;
    }

    .dropdown-item.active {
        background: #e8f0fe;
        color: #1967d2;
    }

    .check {
        color: #1967d2;
        font-size: 0.8rem;
    }

    .space {
        color: #9aa0a6;
        font-size: 0.8rem;
    }
</style>
