<script>
  import LayoutSelector from "./LayoutSelector.svelte";

  export let currentPage = 1;
  export let totalPages = 1;
  export let totalItems = 0;
  export let pageSize = 50;
  export let layoutMode = "vertical-split";
  export let showLayoutSelector = true;
  export let onPageChange = undefined;
  export let onLayoutChange = undefined;

  $: startItem = totalItems > 0 ? (currentPage - 1) * pageSize + 1 : 0;
  $: endItem = Math.min(currentPage * pageSize, totalItems);

  function prevPage() {
    if (currentPage > 1 && onPageChange) {
      onPageChange({ page: currentPage - 1 });
    }
  }

  function nextPage() {
    if (currentPage < totalPages && onPageChange) {
      onPageChange({ page: currentPage + 1 });
    }
  }

  function handleLayoutChange(detail) {
    if (onLayoutChange) {
      onLayoutChange(detail);
    }
  }
</script>

<div class="list-header">
  <div class="left-spacer"></div>

  <div class="right-section">
    <div class="pagination-info">
      <span class="page-text">{startItem}-{endItem} von {totalItems}</span>
      <div class="nav-buttons">
        <button
          class="nav-btn"
          disabled={currentPage === 1}
          onclick={prevPage}
          aria-label="Previous page"
        >
          ‹
        </button>
        <button
          class="nav-btn"
          disabled={currentPage === totalPages}
          onclick={nextPage}
          aria-label="Next page"
        >
          ›
        </button>
      </div>
    </div>

    <div class="layout-slot" class:hidden={!showLayoutSelector}>
      {#if showLayoutSelector}
        <LayoutSelector
          currentMode={layoutMode}
          onChange={handleLayoutChange}
        />
      {/if}
    </div>
  </div>
</div>

<style>
  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 16px;
    background: var(--surface-color);
    border-bottom: 1px solid var(--border-color);
    min-height: 48px;
    flex-shrink: 0;
  }

  .left-spacer {
    flex: 1;
  }

  .right-section {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .pagination-info {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .page-text {
    font-size: 0.75rem;
    color: var(--text-secondary);
    user-select: none;
  }

  .nav-buttons {
    display: flex;
    align-items: center;
    gap: 0;
  }

  .nav-btn {
    width: 36px;
    height: 36px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 50%;
    cursor: pointer;
    font-size: 1.25rem;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background-color 0.1s;
  }

  .nav-btn:hover:not(:disabled) {
    background: var(--hover-color);
    color: var(--text-color);
  }

  .nav-btn:disabled {
    opacity: 0.38;
    cursor: default;
  }

  .layout-slot {
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .layout-slot.hidden {
    display: none;
  }
</style>
