const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

// UI Elements
const elMboxInput = document.getElementById('mbox-input');
const elMbxcInput = document.getElementById('mbxc-input');
const btnBrowseMbox = document.getElementById('btn-browse-mbox');
const btnBrowseMbxc = document.getElementById('btn-browse-mbxc');
const btnConvert = document.getElementById('btn-convert');

const sectionProgress = document.getElementById('progress-section');
const barFill = document.getElementById('progress-fill');
const statPercent = document.getElementById('stat-percent');
const statStatus = document.getElementById('stat-status');
const statElapsed = document.getElementById('stat-elapsed');
const statEta = document.getElementById('stat-eta');
const statSpeed = document.getElementById('stat-speed');
const statSize = document.getElementById('stat-size');
const elLogs = document.getElementById('log-view');
const btnTheme = document.getElementById('theme-toggle');

let totalSize = 0;
let startTime = 0;
let isConverting = false;

// --- Theme Management ---
function initTheme() {
    const saved = localStorage.getItem('theme');
    if (saved) {
        document.documentElement.className = saved;
    } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
        document.documentElement.className = 'dark';
    }
}

btnTheme.onclick = () => {
    const isDark = document.documentElement.classList.contains('dark');
    const next = isDark ? 'light' : 'dark';
    document.documentElement.className = next;
    localStorage.setItem('theme', next);
};

// --- Helper Functions ---
function formatBytes(bytes, decimals = 2) {
    if (bytes === 0) return '0 Bytes';
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

function formatDuration(seconds) {
    if (isNaN(seconds) || seconds === Infinity || seconds < 0) return '---';
    if (seconds < 60) return `${Math.round(seconds)}s`;
    const mins = Math.floor(seconds / 60);
    const secs = Math.round(seconds % 60);
    return `${mins}m ${secs}s`;
}

async function updateMboxPath(path) {
    elMboxInput.value = path;
    if (path) {
        const lastSlash = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
        if (lastSlash !== -1) {
            const folder = path.substring(0, lastSlash + 1);
            const filename = path.substring(lastSlash + 1);
            const dotIdx = filename.lastIndexOf('.');
            const baseName = dotIdx !== -1 ? filename.substring(0, dotIdx) : filename;
            elMbxcInput.value = `${folder}${baseName}.mbxc`;
        } else {
            elMbxcInput.value = `${path}.mbxc`;
        }

        try {
            totalSize = await invoke('get_file_size', { path });
            statSize.textContent = formatBytes(totalSize);
        } catch (e) {
            console.error('Failed to get file size:', e);
        }
    }
}

// --- Event Handlers ---
btnBrowseMbox.onclick = async () => {
    try {
        const selected = await invoke('select_mbox_file');
        if (selected) {
            updateMboxPath(selected);
        }
    } catch (err) {
        alert('Selection failed: ' + err);
    }
};

btnBrowseMbxc.onclick = async () => {
    try {
        const selected = await invoke('select_mbxc_file');
        if (selected) {
            elMbxcInput.value = selected;
        }
    } catch (err) {
        alert('Selection failed: ' + err);
    }
};

btnConvert.onclick = async () => {
    const input = elMboxInput.value.trim();
    const output = elMbxcInput.value.trim();

    if (!input || !output) {
        alert('Please select both input and output files.');
        return;
    }

    try {
        isConverting = true;
        btnConvert.disabled = true;
        sectionProgress.classList.remove('hidden');
        elLogs.classList.add('hidden'); // Hide logs during normal operation
        elLogs.textContent = 'Conversion started...\n';

        startTime = Date.now();
        statStatus.textContent = 'Initializing...';
        barFill.style.width = '0%';
        statPercent.textContent = '0%';

        await invoke('convert_mbox', { input, output });

        statStatus.textContent = 'Complete';
        barFill.style.width = '100%';
        statPercent.textContent = '100%';
        elLogs.classList.remove('hidden');
        elLogs.textContent += '\nConversion successful!';
    } catch (err) {
        statStatus.textContent = 'Error';
        elLogs.classList.remove('hidden');
        elLogs.textContent += `\nERROR: ${err}`;
        alert('Conversion failed: ' + err);
    } finally {
        isConverting = false;
        btnConvert.disabled = false;
    }
};

// --- Progress Listening ---
listen('conversion-progress', (event) => {
    const { bytes, count } = event.payload;

    const now = Date.now();
    const elapsed = (now - startTime) / 1000;

    // Calculate stats
    const currentTotal = totalSize || bytes || 1;
    const percent = (bytes / currentTotal) * 100;
    const speed = bytes / (elapsed || 0.1); // avoid division by zero
    const remaining = Math.max(0, currentTotal - bytes);
    const eta = speed > 0 ? remaining / speed : 0;

    // UI Updates (Overwrite existing stats, no log appending)
    barFill.style.width = `${Math.min(percent, 100).toFixed(2)}%`;
    statPercent.textContent = `${Math.min(percent, 100).toFixed(1)}%`;
    statStatus.textContent = `${count.toLocaleString()} Emails`;
    statElapsed.textContent = formatDuration(elapsed);
    statEta.textContent = formatDuration(eta);
    statSpeed.textContent = `${formatBytes(speed)}/s`;
});

initTheme();
