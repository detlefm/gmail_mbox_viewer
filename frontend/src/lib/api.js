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
    const response = await fetch(`${BASE_URL}/system/info`);
    if (!response.ok) throw new Error("Failed to fetch system info");
    return response.json();
}

export async function selectFile() {
    const response = await fetch(`${BASE_URL}/system/select-file`, { method: "POST" });
    if (!response.ok) throw new Error("Failed to select file");
    const data = await response.json();
    return data.path;
}

export async function selectSaveFile() {
    const response = await fetch(`${BASE_URL}/system/select-save-file`, { method: "POST" });
    if (!response.ok) throw new Error("Failed to select save file");
    const data = await response.json();
    return data.path;
}

export async function convertMbox(mboxPath, mbxcPath) {
    const response = await fetch(`${BASE_URL}/system/convert`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ mbox_path: mboxPath, mbxc_path: mbxcPath })
    });
    if (!response.ok) throw new Error("Conversion failed");
    return response.json();
}

export async function updateSettings(zipPath, browser) {
    const response = await fetch(`${BASE_URL}/system/settings`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ zip_path: zipPath, browser })
    });
    if (!response.ok) throw new Error("Failed to update settings");
    return response.json();
}

export async function selectSettingsFile() {
    const response = await fetch(`${BASE_URL}/system/select-toml`, { method: "POST" });
    if (!response.ok) throw new Error("Failed to select settings file");
    const data = await response.json();
    return data.path;
}

export async function restartWithSettings(settingsPath) {
    const response = await fetch(`${BASE_URL}/system/restart`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ settings_path: settingsPath })
    });
    if (!response.ok) throw new Error("Failed to restart with new settings");
    return response.json();
}
