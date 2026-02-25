<script>
    import { getAttachmentUrl } from "../lib/api";
    import DOMPurify from "dompurify";

    // Force all links to open in a new tab
    DOMPurify.addHook("afterSanitizeAttributes", function (node) {
        if ("target" in node) {
            node.setAttribute("target", "_blank");
            node.setAttribute("rel", "noopener noreferrer");
        }
    });

    export let message = null;

    // Use srcdoc for iframe to display HTML safely-ish
    $: htmlContent = (() => {
        const msg = message;
        if (!msg?.body) return "";

        let content = msg.body;

        // If it's plain text, linkify URLs first
        if (!msg.is_html) {
            const urlRegex =
                /(\b(https?|ftp|file):\/\/[-A-Z0-9+&@#\/%?=~_|!:,.;]*[-A-Z0-9+&@#\/%=~_|])/gi;
            content = content.replace(urlRegex, '<a href="$1">$1</a>');
        } else {
            // Replace cid: references with actual attachment URLs
            if (msg.attachments && msg.attachments.length > 0) {
                msg.attachments.forEach((att) => {
                    if (att.content_id) {
                        // Extract CID from format like <image015.gif@...> or just image015.gif@...
                        const cid = att.content_id.replace(/[<>]/g, "");
                        const url = getAttachmentUrl(msg.id, att.filename);
                        // Escape CID for use in regex
                        const escapedCid = cid.replace(
                            /[.*+?^${}()|[\]\\]/g,
                            "\\$&",
                        );
                        const cidRegex = new RegExp("cid:" + escapedCid, "g");
                        content = content.replace(cidRegex, url);
                    }
                });
            }
        }

        return DOMPurify.sanitize(content, {
            USE_PROFILES: { html: true },
            ADD_ATTR: ["target"],
        });
    })();

    // Helper to get initials for avatar
    function getInitials(from) {
        if (!from) return "?";
        const match = from.match(/([^<]+)/);
        const name = match ? match[1].trim() : from;
        return name.charAt(0).toUpperCase();
    }

    let detailsOpen = false;

    function toggleDetails() {
        detailsOpen = !detailsOpen;
    }

    // Helper to parse name and email
    function parseSender(from) {
        if (!from) return { name: "Unknown", email: "" };
        const emailMatch = from.match(/<(.+?)>/);
        const nameMatch = from.match(/([^<]+)/);
        return {
            name: nameMatch ? nameMatch[1].trim() : from,
            email: emailMatch ? emailMatch[1] : "",
        };
    }

    $: sender = parseSender(message?.from);

    // Helper to get recipient display
    function getRecipientDisplay(to) {
        if (!to) return "unbekannt";
        // Simple split by comma for multiple recipients
        const recipients = to.split(",").filter((r) => r.trim().length > 0);
        if (recipients.length > 1) return "Diverse";

        const r = recipients[0];
        const emailMatch = r.match(/<(.+?)>/);
        const nameMatch = r.match(/([^<]+)/);

        // Return name if available, otherwise email, otherwise the raw string
        return (
            (nameMatch ? nameMatch[1].trim() : null) ||
            (emailMatch ? emailMatch[1] : null) ||
            r
        );
    }

    $: recipientDisplay = getRecipientDisplay(message?.to);

    // Helper to format date according to regional settings
    function formatFullDate(dateStr) {
        if (!dateStr) return "";
        const date = new Date(dateStr);
        if (isNaN(date.getTime())) return dateStr;

        return new Intl.DateTimeFormat(undefined, {
            year: "numeric",
            month: "2-digit",
            day: "2-digit",
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
        }).format(date);
    }

    $: formattedDate = formatFullDate(message?.date);

    // Close dropdown on click outside
    import { onMount, onDestroy } from "svelte";
    function handleClickOutside(event) {
        if (detailsOpen && !event.target.closest(".to-me-container")) {
            detailsOpen = false;
        }
    }

    // Filter attachments to hide inline images from the bottom attachment list,
    // but ONLY if they are actually referenced in the body.
    $: displayedAttachments =
        message?.attachments?.filter((att) => {
            if (!att.content_id) return true;
            const cid = att.content_id.replace(/[<>]/g, "");
            return !message.body?.includes("cid:" + cid);
        }) || [];

    onMount(() => {
        window.addEventListener("click", handleClickOutside);
    });

    onDestroy(() => {
        window.removeEventListener("click", handleClickOutside);
    });
</script>

{#if !message}
    <div class="no-selection">Select a message to view</div>
{:else}
    <div class="message-detail">
        <div class="header">
            <div class="subject-row">
                <div class="subject-container">
                    <h2 class="subject">
                        {message?.subject || "(No Subject)"}
                    </h2>
                    {#if message?.labels && message.labels.length > 0}
                        <div class="labels">
                            {#each message.labels as label}
                                <span class="label-chip">
                                    {label}
                                    <button
                                        class="label-remove"
                                        title="Remove label">×</button
                                    >
                                </span>
                            {/each}
                        </div>
                    {/if}
                </div>
                <div class="header-actions">
                    <button class="icon-btn" title="Print all">
                        <svg viewBox="0 0 24 24" width="20" height="20"
                            ><path
                                fill="currentColor"
                                d="M19 8H5c-1.66 0-3 1.34-3 3v6h4v4h12v-4h4v-6c0-1.66-1.34-3-3-3zm-3 11H8v-5h8v5zm3-7c-.55 0-1-.45-1-1s.45-1 1-1 1 .45 1 1-.45 1-1 1zm-1-9H6v4h12V3z"
                            /></svg
                        >
                    </button>
                    <button class="icon-btn" title="In new window">
                        <svg viewBox="0 0 24 24" width="20" height="20"
                            ><path
                                fill="currentColor"
                                d="M19 19H5V5h7V3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"
                            /></svg
                        >
                    </button>
                </div>
            </div>

            <div class="sender-row">
                <div class="sender-left">
                    <div class="avatar" style="background-color: #f44336;">
                        {getInitials(message.from)}
                    </div>
                    <div class="sender-info">
                        <div class="sender-name-row">
                            <span class="sender-name">{sender.name}</span>
                            {#if sender.email}
                                <span class="sender-email"
                                    >&lt;{sender.email}&gt;</span
                                >
                            {/if}
                        </div>
                        <div class="to-me-container">
                            <button class="to-me-row" onclick={toggleDetails}>
                                <span class="to-me">an {recipientDisplay}</span>
                                <svg
                                    class="dropdown-arrow"
                                    class:open={detailsOpen}
                                    viewBox="0 0 24 24"
                                    width="16"
                                    height="16"
                                    ><path
                                        fill="currentColor"
                                        d="M7 10l5 5 5-5z"
                                    /></svg
                                >
                            </button>

                            {#if detailsOpen}
                                <div class="details-popover">
                                    <table class="details-table">
                                        <tbody>
                                            <tr>
                                                <td class="label">von:</td>
                                                <td>{message?.from}</td>
                                            </tr>
                                            <tr>
                                                <td class="label">an:</td>
                                                <td>{message?.to}</td>
                                            </tr>
                                            <tr>
                                                <td class="label">Datum:</td>
                                                <td>{formattedDate}</td>
                                            </tr>
                                            <tr>
                                                <td class="label">Betreff:</td>
                                                <td>{message?.subject}</td>
                                            </tr>
                                            <tr>
                                                <td class="label">Dateiname:</td
                                                >
                                                <td>{message?.id}</td>
                                            </tr>
                                        </tbody>
                                    </table>
                                </div>
                            {/if}
                        </div>
                    </div>
                </div>
                <div class="sender-right">
                    <span class="date-text">{formattedDate}</span>
                    <div class="action-icons">
                        <button class="icon-btn" title="Star">
                            <svg viewBox="0 0 24 24" width="20" height="20"
                                ><path
                                    fill="currentColor"
                                    d="M12 17.27L18.18 21l-1.64-7.03L22 9.24l-7.19-.61L12 2 9.19 8.63 2 9.24l5.46 4.73L5.82 21z"
                                /></svg
                            >
                        </button>
                        <button class="icon-btn" title="Reply">
                            <svg viewBox="0 0 24 24" width="20" height="20"
                                ><path
                                    fill="currentColor"
                                    d="M10 9V5l-7 7 7 7v-4.1c5 0 8.5 1.6 11 5.1-1-5-4-10-11-11z"
                                /></svg
                            >
                        </button>
                        <button class="icon-btn" title="More">
                            <svg viewBox="0 0 24 24" width="20" height="20"
                                ><path
                                    fill="currentColor"
                                    d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"
                                /></svg
                            >
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <div class="content">
            {#if message?.is_html}
                <iframe
                    title="Message Content"
                    sandbox="allow-popups allow-popups-to-escape-sandbox"
                    srcdoc={htmlContent}
                ></iframe>
            {:else}
                <pre>{@html htmlContent}</pre>
            {/if}
        </div>

        {#if displayedAttachments.length > 0}
            <div class="attachments">
                {#each displayedAttachments as att}
                    <div class="attachment-card">
                        <div class="attachment-thumb">
                            <svg
                                viewBox="0 0 24 24"
                                width="48"
                                height="48"
                                style="color: #5f6368;"
                                ><path
                                    fill="currentColor"
                                    d="M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"
                                /></svg
                            >
                        </div>
                        <div class="attachment-footer">
                            <span class="attachment-name">{att.filename}</span>
                            <a
                                class="attachment-download"
                                href={getAttachmentUrl(
                                    message?.id,
                                    att.filename,
                                )}
                                target="_blank"
                                download
                                title="Download"
                            >
                                <svg viewBox="0 0 24 24" width="16" height="16"
                                    ><path
                                        fill="currentColor"
                                        d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"
                                    /></svg
                                >
                            </a>
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>
{/if}

<style>
    .no-selection {
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100%;
        color: var(--text-secondary);
    }

    .message-detail {
        display: flex;
        flex-direction: column;
        height: 100%;
        background: var(--surface-color);
        overflow-y: auto;
    }

    .header {
        padding: 20px 24px 8px 24px;
    }

    .subject-row {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 24px;
    }

    .subject-container {
        display: flex;
        align-items: center;
        gap: 8px;
        flex-wrap: wrap;
    }

    .subject {
        font-size: 1.375rem;
        font-weight: 400;
        margin: 0;
        color: var(--text-color);
    }

    .labels {
        display: flex;
        gap: 4px;
    }

    .label-chip {
        display: inline-flex;
        align-items: center;
        background: var(--input-bg);
        color: var(--text-secondary);
        padding: 2px 4px 2px 8px;
        border-radius: 4px;
        font-size: 0.75rem;
        height: 18px;
    }

    .label-remove {
        background: none;
        border: none;
        padding: 0 4px;
        cursor: pointer;
        font-size: 14px;
        color: var(--text-secondary);
        display: flex;
        align-items: center;
    }

    .header-actions {
        display: flex;
        gap: 8px;
    }

    .sender-row {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
    }

    .sender-left {
        display: flex;
        gap: 12px;
    }

    .avatar {
        width: 40px;
        height: 40px;
        border-radius: 50%;
        background-color: #f44336;
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        font-weight: 500;
        font-size: 1.25rem;
    }

    .sender-info {
        display: flex;
        flex-direction: column;
    }

    .sender-name-row {
        display: flex;
        align-items: baseline;
        gap: 4px;
    }

    .sender-name {
        font-weight: 700;
        font-size: 0.875rem;
        color: var(--text-color);
    }

    .sender-email {
        font-size: 0.75rem;
        color: var(--text-secondary);
    }

    .to-me-container {
        position: relative;
    }

    .to-me-row {
        display: flex;
        align-items: center;
        font-size: 0.8125rem;
        color: var(--text-secondary);
        cursor: pointer;
        padding: 2px 4px;
        border-radius: 4px;
        width: fit-content;
        background: none;
        border: none;
    }

    .to-me-row:hover {
        background-color: var(--hover-color);
    }

    .dropdown-arrow {
        margin-left: -2px;
        transition: transform 0.2s;
    }

    .dropdown-arrow.open {
        transform: rotate(180deg);
    }

    .details-popover {
        position: absolute;
        top: 100%;
        left: 0;
        z-index: 100;
        background: var(--surface-color);
        border: 1px solid var(--border-strong);
        box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
        padding: 16px;
        border-radius: 8px;
        width: max-content;
        max-width: 500px;
    }

    .details-table {
        font-size: 0.8125rem;
        color: var(--text-color);
        border-collapse: collapse;
    }

    .details-table td {
        padding: 4px 0;
        vertical-align: top;
    }

    .details-table td.label {
        color: var(--text-secondary);
        padding-right: 12px;
        text-align: right;
        width: 80px;
    }

    .sender-right {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .date-text {
        font-size: 0.75rem;
        color: var(--text-secondary);
    }

    .action-icons {
        display: flex;
        gap: 4px;
    }

    .icon-btn {
        background: none;
        border: none;
        padding: 8px;
        border-radius: 50%;
        cursor: pointer;
        color: var(--text-secondary);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .icon-btn:hover {
        background-color: var(--hover-color);
    }

    .attachments {
        padding: 0 24px 24px 76px;
        display: flex;
        gap: 12px;
        flex-wrap: wrap;
    }

    .attachment-card {
        width: 156px;
        border: 1px solid var(--border-strong);
        border-radius: 4px;
        overflow: hidden;
        cursor: pointer;
    }

    .attachment-thumb {
        height: 100px;
        background: var(--input-bg);
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .attachment-footer {
        padding: 8px;
        display: flex;
        justify-content: space-between;
        align-items: center;
        background: var(--surface-color);
        border-top: 1px solid var(--border-strong);
    }

    .attachment-name {
        font-size: 0.75rem;
        color: var(--text-color);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        font-weight: 500;
    }

    .attachment-download {
        color: var(--text-secondary);
        display: flex;
        align-items: center;
    }

    .content {
        flex: 1;
        padding: 8px 24px 24px 76px;
    }

    iframe {
        width: 100%;
        height: 600px;
        border: none;
        background: #fff; /* Email content is usually designed for white */
        border-radius: 8px;
    }

    :root[data-theme="dark"] iframe {
        filter: invert(0.9) hue-rotate(180deg);
        background: #fff;
    }

    pre {
        white-space: pre-wrap;
        font-family: inherit;
        font-size: 0.875rem;
        color: var(--text-color);
        margin: 0;
    }
</style>
