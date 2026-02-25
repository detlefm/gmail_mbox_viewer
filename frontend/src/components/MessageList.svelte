<script>
  export let messages = [];
  export let selectedMessageId = null;
  export let selectedLabel = null;
  export let onSelect = undefined;

  // Filter out selected label from labels array
  function getFilteredLabels(labels, currentLabel) {
    if (!labels || !currentLabel) return labels;
    return labels.filter((label) => label !== currentLabel);
  }

  // Format date to GMail-style (e.g., "12.Feb.2025")
  function formatGmailDate(dateStr) {
    if (!dateStr) return "";
    try {
      const date = new Date(dateStr);
      const now = new Date();

      // Same year - show day.month
      if (date.getFullYear() === now.getFullYear()) {
        return date.toLocaleDateString("de-DE", {
          day: "2-digit",
          month: "short",
        });
      }
      // Different year - show day.month.year
      return date.toLocaleDateString("de-DE", {
        day: "2-digit",
        month: "short",
        year: "numeric",
      });
    } catch (e) {
      return dateStr;
    }
  }

  // Truncate on word boundary
  function truncateOnWordBoundary(text, maxLength = 80) {
    if (!text) return "";
    if (text.length <= maxLength) return text;

    const truncated = text.substring(0, maxLength);
    const lastSpace = truncated.lastIndexOf(" ");

    if (lastSpace > maxLength * 0.7) {
      return truncated.substring(0, lastSpace) + "...";
    }
    return truncated + "...";
  }

  function handleSelect(msg) {
    if (onSelect) {
      onSelect(msg);
    }
  }
</script>

<div class="message-list">
  {#if messages.length === 0}
    <div class="empty-state">No messages found.</div>
  {:else}
    <table>
      <tbody>
        {#each messages as msg}
          <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
          <tr
            class:selected={selectedMessageId === msg.id}
            onclick={() => handleSelect(msg)}
            onkeydown={(e) => e.key === "Enter" && handleSelect(msg)}
            role="button"
            tabindex="0"
          >
            <td class="sender"
              >{msg.sender_name || msg.sender_address || "Unknown"}</td
            >
            <td class="content">
              {#if msg.gmail_labels}
                {#each getFilteredLabels(msg.gmail_labels, selectedLabel) as label}
                  <span class="label-chip">{label}</span>
                {/each}
              {/if}
              <span class="subject">{msg.subject || "(No Subject)"}</span>
              {#if msg.snippet}
                <span class="snippet"
                  >{truncateOnWordBoundary(msg.snippet, 100)}</span
                >
              {/if}
            </td>
            <td class="attachment">
              {#if msg.has_attachment}
                <span class="attachment-icon" title="Has attachment">📎</span>
              {/if}
            </td>
            <td class="date">{formatGmailDate(msg.date_sent_iso)}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .message-list {
    flex: 1;
    overflow-y: auto;
    background: var(--surface-color);
  }

  .empty-state {
    padding: 2rem;
    text-align: center;
    color: var(--text-secondary);
    font-size: 0.875rem;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    table-layout: fixed;
  }

  tr {
    border-bottom: 1px solid var(--border-color);
    cursor: pointer;
    height: 40px;
    background: var(--surface-color);
    transition: box-shadow 0.1s;
  }

  tr:hover {
    box-shadow:
      inset 1px 0 0 var(--border-strong),
      inset -1px 0 0 var(--border-strong),
      0 1px 2px 0 rgba(0, 0, 0, 0.2),
      0 1px 3px 1px rgba(0, 0, 0, 0.1);
    z-index: 1;
    position: relative;
  }

  tr.selected {
    background: var(--sidebar-active);
  }

  td {
    padding: 0 8px;
    font-size: 0.875rem;
    color: var(--text-color);
  }

  .sender {
    width: 200px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    padding-left: 16px;
  }

  .content {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .label-chip {
    display: inline-block;
    background: var(--input-bg);
    color: var(--text-secondary);
    padding: 0 4px;
    border-radius: 4px;
    font-size: 0.75rem;
    margin-right: 6px;
    height: 18px;
    line-height: 18px;
    border: 1px solid var(--border-strong);
  }

  .subject {
    font-weight: 700;
    margin-right: 8px;
  }

  .snippet {
    color: var(--text-secondary);
    font-weight: normal;
  }

  .attachment {
    width: 40px;
    text-align: center;
  }

  .attachment-icon {
    font-size: 1rem;
    color: var(--text-secondary);
    opacity: 0.7;
  }

  .date {
    width: 80px;
    text-align: right;
    color: var(--text-secondary);
    font-size: 0.75rem;
    white-space: nowrap;
    padding-right: 16px;
  }
</style>
