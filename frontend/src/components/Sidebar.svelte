<script>
  export let labels = [];
  export let selectedLabel = "";
  export let onSelect = undefined;

  // Track expanded state by full path
  let expandedPaths = new Set();

  // Standard GMail labels in German
  const standardLabels = [
    { name: "Posteingang", icon: "inbox" },
    { name: "Zurückgestellt", icon: "schedule" },
    { name: "Gesendet", icon: "send" },
    { name: "Entwürfe", icon: "insert_drive_file" },
    { name: "Wichtig", icon: "label_important" },
    { name: "Alle Mails", icon: "mail" },
    { name: "Spam", icon: "report" },
    { name: "Papierkorb", icon: "delete" },
  ];

  // Helper to categorize labels
  $: categorized = (() => {
    const standard = [];
    const userRaw = [];

    // Sort standard labels based on the predefined order
    const standardNames = standardLabels.map((l) => l.name);

    // Filter labels into standard and user
    labels.forEach((label) => {
      if (standardNames.includes(label)) {
        standard.push(label);
      } else {
        userRaw.push(label);
      }
    });

    // Re-order standard labels based on predefined list
    const sortedStandard = standardLabels
      .filter((sl) => labels.includes(sl.name))
      .map((sl) => ({ name: sl.name, icon: sl.icon }));

    // Build tree for user labels
    const userTree = buildTree(userRaw);

    return { standard: sortedStandard, user: userTree };
  })();

  function customSort(a, b) {
    const aVal = String(a).toLowerCase();
    const bVal = String(b).toLowerCase();

    // Explicitly handle underscore to ensure it comes before 'A'
    if (aVal.startsWith("_") && !bVal.startsWith("_")) return -1;
    if (!aVal.startsWith("_") && bVal.startsWith("_")) return 1;

    return aVal.localeCompare(bVal);
  }

  function buildTree(labelList) {
    const root = [];
    const map = {};

    // Sort the list before building the tree
    [...labelList].sort(customSort).forEach((label) => {
      const parts = label.split("/");
      let currentLevel = root;
      let path = "";

      parts.forEach((part, index) => {
        path = path ? `${path}/${part}` : part;

        if (!map[path]) {
          const newNode = {
            name: part,
            fullPath: path,
            children: [],
          };
          map[path] = newNode;
          currentLevel.push(newNode);

          // Sort children of current level after adding
          currentLevel.sort((a, b) => customSort(a.name, b.name));
        }
        currentLevel = map[path].children;
      });
    });

    return root;
  }

  function handleSelect(label) {
    if (onSelect) {
      onSelect(label);
    }
  }

  function toggleExpand(path, event) {
    event.stopPropagation();
    if (expandedPaths.has(path)) {
      expandedPaths.delete(path);
    } else {
      expandedPaths.add(path);
    }
    expandedPaths = new Set(expandedPaths); // Trigger reactivity
  }

  // Icons as SVG components or paths
  const icons = {
    inbox: `<path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H5v-3h3.56c.69 1.19 1.97 2 3.44 2s2.75-.81 3.44-2H19v3zm0-5h-4.95c-.45.91-1.4 1.5-2.5 1.5s-2.05-.59-2.5-1.5H5V5h14v9z"/>`,
    send: `<path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>`,
    insert_drive_file: `<path d="M6 2c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6H6zm7 7V3.5L18.5 9H13z"/>`,
    mail: `<path d="M20 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V6c0-1.1-.9-2-2-2zm0 4l-8 5-8-5V6l8 5 8-5v2z"/>`,
    report: `<path d="M15.73 3H8.27L3 8.27v7.46L8.27 21h7.46L21 15.73V8.27L15.73 3zM12 17.3c-.72 0-1.3-.58-1.3-1.3s.58-1.3 1.3-1.3 1.3.58 1.3 1.3-.58 1.3-1.3 1.3zm1-4.3h-2V7h2v6z"/>`,
    delete: `<path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>`,
    label_important: `<path d="M3.5 18.99l11 .01c.67 0 1.27-.33 1.63-.84L20.5 12l-4.37-6.16c-.36-.51-.96-.84-1.63-.84l-11 .01L8.34 12 3.5 18.99z"/>`,
    schedule: `<path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z"/>`,
    folder_label: `<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/>`,
    arrow_right: `<path d="M10 17l5-5-5-5v10z"/>`,
    arrow_down: `<path d="M7 10l5 5 5-5z"/>`,
  };
</script>

{#snippet treeNode(node, level)}
  {@const isExpanded = expandedPaths.has(node.fullPath)}
  <li class="tree-item" class:active={node.fullPath === selectedLabel}>
    <div class="row-container" style="padding-left: {level * 20 + 12}px">
      <span
        class="tree-arrow"
        class:invisible={node.children.length === 0}
        class:expanded={isExpanded}
        onclick={(e) => toggleExpand(node.fullPath, e)}
        role="button"
        tabindex="0"
        onkeydown={(e) => e.key === "Enter" && toggleExpand(node.fullPath, e)}
      >
        <svg viewBox="0 0 24 24"
          >{@html isExpanded ? icons.arrow_down : icons.arrow_right}</svg
        >
      </span>
      <button onclick={() => handleSelect(node.fullPath)} title={node.fullPath}>
        <span class="icon">
          <svg viewBox="0 0 24 24" style="color: #5f6368;"
            ><path
              d="M12 21.35l-1.45-1.32C5.4 15.36 2 12.28 2 8.5 2 5.42 4.42 3 7.5 3c1.74 0 3.41.81 4.5 2.09C13.09 3.81 14.76 3 16.5 3 19.58 3 22 5.42 22 8.5c0 3.78-3.4 6.86-8.55 11.54L12 21.35z"
            /></svg
          >
          <!-- Generic label icon -->
        </span>
        <span class="label-name">{node.name}</span>
      </button>
    </div>
    {#if node.children.length > 0 && isExpanded}
      <ul class="tree-list">
        {#each node.children as child}
          {@render treeNode(child, level + 1)}
        {/each}
      </ul>
    {/if}
  </li>
{/snippet}

<aside class="sidebar">
  <ul class="nav-list">
    {#each categorized.standard as item}
      <li class="nav-item" class:active={item.name === selectedLabel}>
        <button onclick={() => handleSelect(item.name)} title={item.name}>
          <span class="icon">
            <svg viewBox="0 0 24 24">{@html icons[item.icon]}</svg>
          </span>
          <span class="label-name">{item.name}</span>
        </button>
      </li>
    {/each}
  </ul>

  <div class="user-labels-section">
    <div class="section-header">
      <span class="header-title">Labels</span>
      <button class="add-btn" title="Neues Label">+</button>
    </div>

    <ul class="tree-list">
      {#each categorized.user as node}
        {@render treeNode(node, 0)}
      {/each}
    </ul>
  </div>
</aside>

<style>
  .sidebar {
    width: 250px;
    background: var(--bg-color);
    padding: 8px 0;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .nav-list,
  .tree-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .nav-item,
  .tree-item {
    margin: 0 12px 0 0;
  }

  .nav-item button,
  .row-container button {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    background: transparent;
    border: none;
    padding: 0 12px 0 26px;
    height: 32px;
    cursor: pointer;
    border-top-right-radius: 16px;
    border-bottom-right-radius: 16px;
    color: var(--text-color);
    font-size: 0.875rem;
    text-align: left;
    transition: background-color 0.1s;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  .row-container {
    display: flex;
    align-items: center;
    border-top-right-radius: 16px;
    border-bottom-right-radius: 16px;
    cursor: pointer;
  }

  .row-container button {
    padding-left: 8px;
  }

  .row-container:hover,
  .nav-item button:hover {
    background-color: var(--hover-color);
  }

  .nav-item.active button,
  .tree-item.active > .row-container {
    background-color: var(--sidebar-active);
    color: var(--sidebar-active-text);
    font-weight: 500;
  }

  .icon {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon svg {
    width: 20px;
    height: 20px;
    fill: currentColor;
  }

  .tree-arrow {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .tree-arrow svg {
    width: 20px;
    height: 20px;
    fill: currentColor;
    transition: transform 0.2s;
  }

  .tree-arrow.expanded svg {
    transform: rotate(90deg);
  }

  .tree-arrow.invisible {
    visibility: hidden;
  }

  .user-labels-section {
    margin-top: 4px;
    padding-top: 4px;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 16px 4px 24px;
    color: var(--text-secondary);
  }

  .header-title {
    font-size: 0.875rem;
    font-weight: 500;
  }

  .add-btn {
    background: transparent;
    border: none;
    cursor: pointer;
    font-size: 1.25rem;
    color: var(--text-secondary);
    padding: 0 4px;
    line-height: 1;
    border-radius: 4px;
  }

  .add-btn:hover {
    background-color: var(--hover-color);
  }

  .label-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
