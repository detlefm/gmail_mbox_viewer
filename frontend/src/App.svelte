<script>
  import { onMount } from "svelte";
  import Header from "./components/Header.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import MessageList from "./components/MessageList.svelte";
  import MessageDetail from "./components/MessageDetail.svelte";
  import SearchPopup from "./components/SearchPopup.svelte";
  import ListHeader from "./components/ListHeader.svelte";
  import Management from "./components/Management.svelte";
  import * as api from "./lib/api";

  let labels = [];
  let messages = [];
  let totalMessages = 0;

  let selectedLabel = "INBOX";
  let selectedMessage = null;
  let selectedMessageId = null;

  let showSearchPopup = false;
  let loading = false;
  let error = null;

  // Instance tracking for backend restarts
  let lastInstanceId = null;
  let lastZipPath = null;
  let settingsPath = "";
  let dbLoaded = false;
  let initLoading = true;
  let isBackendLoading = true;
  let hasInitialized = false;
  let backendLoadMsg = "Daten werden geladen...";
  let viewMode = "viewer"; // 'viewer' or 'management'
  let convertStatus = {
    is_running: false,
    progress_percent: 0,
    current_message: 0,
  };

  $: archiveName = getArchiveName(lastZipPath);

  function getArchiveName(path) {
    if (!path) return "Keine Datenquelle";
    const parts = path.split(/[/\\]/);
    return parts[parts.length - 1] || "Keine Datenquelle";
  }
  async function checkServerStatus() {
    try {
      const info = await api.getSystemInfo();
      const backendReady = !info.is_loading;

      // If instance ID changed while ready, trigger a full data refresh
      // This replaces the old yellow banner.
      if (
        hasInitialized &&
        !isBackendLoading &&
        lastInstanceId &&
        info.instance_id !== lastInstanceId
      ) {
        console.log("Backend instance changed, refreshing data...");
        loadLabels().then(() => {
          handleAutoDetectLabel();
        });
      }

      const prevLoading = isBackendLoading;
      lastInstanceId = info.instance_id;
      lastZipPath = info.zip_path;
      settingsPath = info.settings_path || "";
      dbLoaded = info.db_loaded;
      isBackendLoading = !backendReady;

      // Update initialization state
      if (backendReady && !hasInitialized) {
        hasInitialized = true;
      }

      // Detect transition from loading to ready
      if (prevLoading && backendReady) {
        // Successful transition: wait a tiny bit to ensure backend is fully ready
        setTimeout(async () => {
          await loadLabels();
          handleAutoDetectLabel();
        }, 100);
      }

      if (initLoading) {
        initLoading = false;
        if (!dbLoaded && backendReady) {
          viewMode = "management";
        }
      }
    } catch (e) {
      // Ignore errors (e.g. during restart)
    }
  }

  async function checkConvertStatus() {
    try {
      const status = await api.getConvertStatus();
      convertStatus = status;
    } catch (e) {
      // Ignore errors
    }
  }

  // Layout mode: 'no-split', 'vertical-split', 'horizontal-split'
  let layoutMode =
    typeof localStorage !== "undefined"
      ? localStorage.getItem("layoutMode") || "vertical-split"
      : "vertical-split";

  // Pagination state
  let currentPage = 1;
  let pageSize = 50; // Adaptive: Desktop 50, Mobile 15

  // Resizable panel widths
  let listWidthPercent =
    typeof localStorage !== "undefined"
      ? parseFloat(localStorage.getItem("listWidthPercent")) || 40
      : 40;
  let listHeightPercent =
    typeof localStorage !== "undefined"
      ? parseFloat(localStorage.getItem("listHeightPercent")) || 40
      : 40;
  let isResizing = false;
  let resizeType = null; // 'vertical' or 'horizontal'
  let startX = 0;
  let startY = 0;
  let startSize = 0;

  // Theme state
  let theme =
    typeof localStorage !== "undefined"
      ? localStorage.getItem("theme") || "auto"
      : "auto";

  function applyTheme() {
    let activeTheme = theme;
    if (theme === "auto") {
      activeTheme = window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";
    }
    document.documentElement.setAttribute("data-theme", activeTheme);
  }

  function toggleTheme() {
    if (theme === "light") theme = "dark";
    else if (theme === "dark") theme = "light";
    else {
      // If currently auto, switch to the opposite of current system
      theme = window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "light"
        : "dark";
    }

    if (typeof localStorage !== "undefined") {
      localStorage.setItem("theme", theme);
    }
    applyTheme();
  }

  // Listen for system theme changes if in 'auto' mode
  if (typeof window !== "undefined") {
    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", () => {
        if (theme === "auto") applyTheme();
      });
  }

  // Determine adaptive page size
  function getAdaptivePageSize() {
    return window.innerWidth < 768 ? 15 : 50;
  }

  async function loadLabels(retryOn503 = true) {
    try {
      labels = await api.getLabels();
      error = null;
    } catch (e) {
      if (
        (retryOn503 && e.message.includes("503")) ||
        e.message.toLowerCase().includes("service unavailable")
      ) {
        // Retry once after 500ms
        setTimeout(() => loadLabels(false), 500);
        return;
      }
      if (!isBackendLoading) {
        console.error(e);
        error = "Failed to load labels";
      }
    }
  }

  async function loadMessages(query = {}, resetPage = true, retryOn503 = true) {
    loading = true;
    error = null;
    try {
      if (resetPage) {
        currentPage = 1;
      }
      pageSize = getAdaptivePageSize();
      const res = await api.searchMessages({
        ...query,
        limit: pageSize,
        offset: (currentPage - 1) * pageSize,
      });
      messages = res.messages;
      totalMessages = res.total;
      error = null;
    } catch (e) {
      if (
        retryOn503 &&
        (e.message.includes("503") ||
          e.message.toLowerCase().includes("service unavailable"))
      ) {
        // Retry once after 500ms
        setTimeout(() => loadMessages(query, resetPage, false), 500);
        return;
      }
      if (!isBackendLoading) {
        console.error(e);
        error = "Failed to load messages";
      }
    } finally {
      loading = false;
    }
  }

  function handleLabelSelect(label) {
    selectedLabel = label;
    selectedMessage = null;
    selectedMessageId = null;
    loadMessages({ label: selectedLabel });

    // Save selection
    if (typeof localStorage !== "undefined" && settingsPath) {
      localStorage.setItem(`lastLabel:${settingsPath}`, label);
    }
  }

  function handleSimpleSearch(detail) {
    const q = detail.any;
    selectedMessage = null;
    selectedMessageId = null;
    loadMessages({ any: q });
  }

  function handleAdvancedSearch(q) {
    selectedLabel = q.label || "";
    selectedMessage = null;
    selectedMessageId = null;
    loadMessages(q);
    showSearchPopup = false;

    // Save selection
    if (typeof localStorage !== "undefined" && settingsPath && q.label) {
      localStorage.setItem(`lastLabel:${settingsPath}`, q.label);
    }
  }

  async function handleMessageSelect(summary) {
    selectedMessageId = summary.id;

    try {
      const fullMsg = await api.getMessage(selectedMessageId);
      selectedMessage = { ...fullMsg, id: selectedMessageId };
    } catch (e) {
      console.error(e);
      alert("Failed to load message content");
    }
  }

  function handleLayoutChange(detail) {
    layoutMode = detail.mode;
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("layoutMode", layoutMode);
    }
  }

  function handlePageChange(detail) {
    currentPage = detail.page;
    loadMessages({ label: selectedLabel }, false);
    // Scroll to top of message list
    const listContainer = document.querySelector(".list-container");
    if (listContainer) {
      listContainer.scrollTop = 0;
    }
  }

  function goBackToList() {
    selectedMessage = null;
    selectedMessageId = null;
  }

  // Message navigation for no-split mode
  $: selectedMessageIndex = messages.findIndex(
    (m) => m.id === selectedMessageId,
  );

  async function navigateToMessage(index) {
    if (index >= 0 && index < messages.length) {
      const msg = messages[index];
      selectedMessageId = msg.id;
      try {
        const fullMsg = await api.getMessage(selectedMessageId);
        selectedMessage = { ...fullMsg, id: selectedMessageId };
      } catch (e) {
        console.error(e);
        alert("Failed to load message content");
      }
    }
  }

  function prevMessage() {
    if (selectedMessageIndex > 0) {
      navigateToMessage(selectedMessageIndex - 1);
    }
  }

  function nextMessage() {
    if (selectedMessageIndex < messages.length - 1) {
      navigateToMessage(selectedMessageIndex + 1);
    }
  }

  // Resize handling
  function startResize(e, type) {
    e.preventDefault();
    isResizing = true;
    resizeType = type;
    startX = e.clientX;
    startY = e.clientY;
    startSize = type === "vertical" ? listWidthPercent : listHeightPercent;

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", stopResize);
    document.body.style.cursor =
      type === "vertical" ? "col-resize" : "row-resize";
    document.body.style.userSelect = "none";
  }

  function handleMouseMove(e) {
    if (!isResizing) return;
    e.preventDefault();

    const availableWidth = window.innerWidth - 250; // Subtract sidebar
    const availableHeight = window.innerHeight - 64; // Subtract header

    if (resizeType === "vertical") {
      const startWidthPx = (startSize / 100) * availableWidth;
      const deltaPx = e.clientX - startX;
      const newWidthPx = startWidthPx + deltaPx;
      listWidthPercent = Math.max(
        20,
        Math.min(80, (newWidthPx / availableWidth) * 100),
      );
    } else if (resizeType === "horizontal") {
      const startHeightPx = (startSize / 100) * availableHeight;
      const deltaPx = e.clientY - startY;
      const newHeightPx = startHeightPx + deltaPx;
      listHeightPercent = Math.max(
        20,
        Math.min(80, (newHeightPx / availableHeight) * 100),
      );
    }
  }

  function stopResize() {
    isResizing = false;
    resizeType = null;
    document.removeEventListener("mousemove", handleMouseMove);
    document.removeEventListener("mouseup", stopResize);
    document.body.style.cursor = "";
    document.body.style.userSelect = "";

    // Save panel sizes
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("listWidthPercent", listWidthPercent.toString());
      localStorage.setItem("listHeightPercent", listHeightPercent.toString());
    }
  }

  function customSort(a, b) {
    const aVal = String(a).toLowerCase();
    const bVal = String(b).toLowerCase();
    if (aVal.startsWith("_") && !bVal.startsWith("_")) return -1;
    if (!aVal.startsWith("_") && bVal.startsWith("_")) return 1;
    return aVal.localeCompare(bVal);
  }

  function handleAutoDetectLabel() {
    const standardLabelNames = [
      "Posteingang",
      "Zurückgestellt",
      "Gesendet",
      "Entwürfe",
      "Wichtig",
      "Alle Mails",
      "Spam",
      "Papierkorb",
    ];

    let initialLabel = "";

    // Try to load last used label for this settings file
    if (typeof localStorage !== "undefined" && settingsPath) {
      const saved = localStorage.getItem(`lastLabel:${settingsPath}`);
      if (saved && labels.includes(saved)) {
        selectedLabel = saved;
        loadMessages({ label: saved });
        return;
      }
    }

    if (labels.includes("Posteingang")) {
      initialLabel = "Posteingang";
    } else {
      // Find first user-defined label
      const userLabels = labels.filter((l) => !standardLabelNames.includes(l));
      if (userLabels.length > 0) {
        userLabels.sort(customSort);
        initialLabel = userLabels[0];
      } else {
        initialLabel = labels[0] || "";
      }
    }

    selectedLabel = initialLabel;
    loadMessages({ label: initialLabel });
  }

  onMount(async () => {
    applyTheme();

    // Initial status check - this will set isBackendLoading correctly
    await checkServerStatus();

    // If already finished loading, do initial data fetch
    if (!isBackendLoading) {
      await loadLabels();
      handleAutoDetectLabel();
    }

    // Start polling for backend changes
    setInterval(checkServerStatus, 2000);
    setInterval(checkConvertStatus, 2000);
  });

  $: {
    // If backend is loading, we could increase polling frequency here
    // but the 2s interval above is generally sufficient.
  }
</script>

<div class="app-container">
  {#if isBackendLoading}
    <div class="backend-load-overlay">
      <div class="loader-content">
        <div class="spinner"></div>
        <h2>{backendLoadMsg}</h2>
        <p>Dies kann bei großen Archiven einen Moment dauern.</p>
      </div>
    </div>
  {/if}

  <Header
    onSearch={handleSimpleSearch}
    onOpenAdvanced={() => (showSearchPopup = true)}
    onOpenManagement={() => (viewMode = "management")}
    onOpenViewer={() => (viewMode = "viewer")}
    {viewMode}
    onToggleTheme={toggleTheme}
    {theme}
    {archiveName}
  />

  {#if convertStatus.is_running}
    <div class="global-progress">
      <div class="progress-track">
        <div
          class="progress-bar"
          style:width={`${convertStatus.progress_percent}%`}
        ></div>
      </div>
      <div class="progress-label">
        Konvertierung läuft: {convertStatus.progress_percent}% ({convertStatus.current_message}
        Nachrichten)
      </div>
      <button class="abort-link" on:click={() => api.abortConvert()}
        >Abbrechen</button
      >
    </div>
  {/if}

  <div class="main-layout">
    {#if viewMode === "management"}
      <Management
        onReload={() => {
          viewMode = "viewer";
          isBackendLoading = true;
          // No full page reload needed, polling will take over
        }}
        onClose={() => (viewMode = "viewer")}
      />
    {:else}
      <div class="sidebar-container">
        <Sidebar {labels} {selectedLabel} onSelect={handleLabelSelect} />
      </div>

      <div class="main-content">
        <!-- Header Panel - Full width above content area -->
        {#if layoutMode === "no-split" && selectedMessage}
          <div class="header-panel">
            <button class="back-btn" on:click={goBackToList}>
              ← Back to list
            </button>
            <div class="right-section">
              <div class="pagination-info">
                <span class="page-text"
                  >{selectedMessageIndex + 1} von {messages.length}</span
                >
                <div class="nav-buttons">
                  <button
                    class="nav-btn"
                    disabled={selectedMessageIndex <= 0}
                    on:click={prevMessage}
                    aria-label="Previous message"
                  >
                    ‹
                  </button>
                  <button
                    class="nav-btn"
                    disabled={selectedMessageIndex >= messages.length - 1}
                    on:click={nextMessage}
                    aria-label="Next message"
                  >
                    ›
                  </button>
                </div>
              </div>
              <div class="layout-slot-placeholder"></div>
            </div>
          </div>
        {:else}
          <ListHeader
            {currentPage}
            totalPages={Math.ceil(totalMessages / pageSize)}
            {pageSize}
            totalItems={totalMessages}
            {layoutMode}
            showLayoutSelector={true}
            onPageChange={handlePageChange}
            onLayoutChange={handleLayoutChange}
          />
        {/if}

        <div
          class="content-area"
          class:horizontal={layoutMode === "horizontal-split"}
        >
          <!-- List Panel -->
          {#if layoutMode === "no-split" ? !selectedMessage : true}
            <div
              class="list-panel"
              class:no-split={layoutMode === "no-split"}
              style:width={layoutMode === "vertical-split"
                ? `${listWidthPercent}%`
                : null}
              style:height={layoutMode === "horizontal-split"
                ? `${listHeightPercent}%`
                : null}
            >
              <div class="list-container">
                {#if loading || initLoading}
                  <div class="loading">Loading...</div>
                {:else if error}
                  <div class="error">{error}</div>
                {:else if !dbLoaded}
                  <div class="setup-notice">
                    <div class="setup-info">
                      <h3>Keine Datenquelle geladen</h3>
                      <p>
                        Wähle eine .mbxc Datei in den Einstellungen aus oder
                        konvertiere eine MBOX Datei.
                      </p>
                      <button
                        class="setup-btn"
                        on:click={() => (viewMode = "management")}
                      >
                        Zu den Einstellungen
                      </button>
                    </div>
                  </div>
                {:else}
                  <MessageList
                    {messages}
                    {selectedMessageId}
                    {selectedLabel}
                    onSelect={handleMessageSelect}
                  />
                {/if}
              </div>
            </div>
          {/if}

          <!-- Resize Handle -->
          {#if layoutMode !== "no-split"}
            <div
              class="resize-handle"
              class:vertical={layoutMode === "vertical-split"}
              class:horizontal={layoutMode === "horizontal-split"}
              on:mousedown={(e) =>
                startResize(
                  e,
                  layoutMode === "vertical-split" ? "vertical" : "horizontal",
                )}
              role="slider"
              tabindex="0"
              aria-label="Resize panel"
              aria-valuemin="20"
              aria-valuemax="80"
              aria-valuenow={layoutMode === "vertical-split"
                ? listWidthPercent
                : listHeightPercent}
            ></div>
          {/if}

          <!-- Detail Panel -->
          {#if layoutMode === "no-split" ? selectedMessage : true}
            <div
              class="detail-panel"
              class:no-split={layoutMode === "no-split"}
              style:width={layoutMode === "vertical-split"
                ? `${100 - listWidthPercent}%`
                : null}
              style:height={layoutMode === "horizontal-split"
                ? `${100 - listHeightPercent}%`
                : null}
            >
              <div class="detail-container">
                <MessageDetail message={selectedMessage} />
              </div>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  {#if showSearchPopup}
    <SearchPopup
      {labels}
      onClose={() => (showSearchPopup = false)}
      onSearch={handleAdvancedSearch}
    />
  {/if}
</div>

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }

  .main-layout {
    display: flex;
    flex: 1;
    overflow: hidden;
    position: relative;
  }

  .sidebar-container {
    width: 250px;
    flex-shrink: 0;
    overflow-y: auto;
    border-right: 1px solid var(--border-strong);
    background: var(--surface-color);
  }

  .main-content {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }

  .header-panel {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px;
    background: var(--bg-color);
    border-bottom: 1px solid var(--border-strong);
    min-height: 48px;
    flex-shrink: 0;
  }

  .right-section {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .layout-slot-placeholder {
    min-width: 80px;
  }

  .content-area {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .content-area.horizontal {
    flex-direction: column;
  }

  .list-panel {
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    overflow: hidden;
  }

  .list-container {
    flex: 1;
    overflow-y: auto;
    background: var(--surface-color);
    display: flex;
    flex-direction: column;
  }

  .detail-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--surface-color);
    flex-shrink: 0;
  }

  .pagination-info {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .page-text {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .nav-buttons {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .nav-btn {
    min-width: 28px;
    height: 28px;
    padding: 0;
    background: transparent;
    border: 1px solid var(--border-strong);
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.1s;
  }

  .nav-btn:hover:not(:disabled) {
    background: var(--hover-color);
    border-color: var(--accent-color);
    color: var(--accent-color);
  }

  .nav-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .detail-container {
    flex: 1;
    overflow-y: auto;
  }

  .back-btn {
    background: transparent;
    border: 1px solid var(--border-strong);
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9rem;
    color: var(--text-secondary);
    transition: background 0.2s;
  }

  .back-btn:hover {
    background: var(--hover-color);
  }

  .resize-handle {
    background: transparent;
    position: relative;
    z-index: 10;
    flex-shrink: 0;
  }

  .resize-handle.vertical {
    width: 6px;
    cursor: col-resize;
    border-left: 1px solid transparent;
    border-right: 1px solid transparent;
  }

  .resize-handle.vertical:hover,
  .resize-handle.vertical:active {
    background: #dadce0;
    border-left: 1px solid #dadce0;
    border-right: 1px solid #dadce0;
  }

  .resize-handle.horizontal {
    height: 6px;
    cursor: row-resize;
    width: 100%;
    border-top: 1px solid transparent;
    border-bottom: 1px solid transparent;
  }

  .resize-handle.horizontal:hover,
  .resize-handle.horizontal:active {
    background: var(--border-strong);
    border-top: 1px solid var(--border-strong);
    border-bottom: 1px solid var(--border-strong);
  }

  .loading,
  .error {
    color: var(--accent-color);
    padding: 2rem;
    text-align: center;
    background: var(--surface-color);
    border: 1px solid var(--accent-color);
    border-radius: 8px;
    margin: 1rem;
  }

  .setup-notice {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    background: var(--surface-color);
  }

  .setup-info {
    text-align: center;
    max-width: 400px;
    background: var(--bg-color);
    padding: 2rem;
    border-radius: 12px;
    border: 1px solid var(--border-color);
    box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1);
  }

  .setup-info h3 {
    margin-top: 0;
    color: var(--text-color);
  }

  .setup-info p {
    color: var(--text-secondary);
    margin-bottom: 1.5rem;
    font-size: 0.9rem;
  }

  .setup-btn {
    background: var(--accent-color);
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 600;
  }

  .global-progress {
    background: var(--surface-color);
    border-bottom: 1px solid var(--border-strong);
    padding: 10px 20px;
    display: flex;
    align-items: center;
    gap: 15px;
    z-index: 100;
  }

  .progress-track {
    flex: 1;
    height: 8px;
    background: var(--input-bg);
    border-radius: 4px;
    overflow: hidden;
  }

  .progress-bar {
    height: 100%;
    background: var(--accent-color);
    transition: width 0.3s ease;
  }

  .progress-label {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .abort-link {
    background: transparent;
    border: 1px solid #ef4444;
    color: #ef4444;
    padding: 4px 10px;
    border-radius: 4px;
    font-size: 0.75rem;
    cursor: pointer;
    font-weight: bold;
  }

  .abort-link:hover {
    background: #ef4444;
    color: white;
  }

  /* Dynamic styles based on layout mode */
  .list-panel.no-split {
    flex: 1;
    width: 100% !important;
    height: 100% !important;
  }

  .detail-panel.no-split {
    flex: 1;
    width: 100% !important;
    height: 100% !important;
  }

  @media (max-width: 768px) {
    .sidebar-container {
      display: none;
    }

    .list-panel,
    .detail-panel {
      width: 100% !important;
      height: auto !important;
    }
  }
  .backend-load-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(var(--bg-color-rgb, 255, 255, 255), 0.9);
    backdrop-filter: blur(4px);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 2000;
    color: var(--text-primary);
  }

  .loader-content {
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
  }

  .spinner {
    width: 48px;
    height: 48px;
    border: 4px solid var(--border-strong);
    border-top: 4px solid var(--accent-color);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  @media (prefers-color-scheme: dark) {
    .backend-load-overlay {
      background: rgba(0, 0, 0, 0.85);
    }
  }
</style>
