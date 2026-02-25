const BASE_URL = '/api';

export async function getLabels() {
    const res = await fetch(`${BASE_URL}/labels`);
    if (!res.ok) throw new Error('Failed to fetch labels');
    return res.json();
}

export async function searchMessages(query = {}) {
    const res = await fetch(`${BASE_URL}/query`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(query)
    });
    if (!res.ok) throw new Error('Failed to search messages');
    return res.json();
}

export async function getMessage(id) {
    const res = await fetch(`${BASE_URL}/messages/${encodeURIComponent(id)}`);
    if (!res.ok) throw new Error('Failed to fetch message');
    return res.json();
}

export function getAttachmentUrl(messageId, filename) {
    return `${BASE_URL}/messages/${encodeURIComponent(messageId)}/attachment/${encodeURIComponent(filename)}`;
}

export async function getSystemInfo() {
    const res = await fetch(`${BASE_URL}/system/info`);
    if (!res.ok) throw new Error('Failed to fetch system info');
    return res.json();
}
