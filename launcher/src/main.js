const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const elSettingsPath = document.getElementById('settings-path');
const elMbxcPath = document.getElementById('mbxc-path');
const elPortInput = document.getElementById('port-input');
const elBrowserInput = document.getElementById('browser-input');
const elStatus = document.getElementById('status-display');
const elLogs = document.getElementById('log-container');
const editActions = document.getElementById('edit-actions');
const portEditActions = document.getElementById('port-edit-actions');
const browserEditActions = document.getElementById('browser-edit-actions');
const portWarning = document.getElementById('port-warning');

const btnRestart = document.getElementById('btn-restart');
const btnSelectFile = document.getElementById('btn-select-file');
const btnApplyEdit = document.getElementById('btn-apply-edit');
const btnCancelEdit = document.getElementById('btn-cancel-edit');
const btnApplyPort = document.getElementById('btn-apply-port');
const btnCancelPort = document.getElementById('btn-cancel-port');
const btnApplyBrowser = document.getElementById('btn-apply-browser');
const btnCancelBrowser = document.getElementById('btn-cancel-browser');
const btnCopy = document.getElementById('btn-copy-logs');
const btnOpenConverter = document.getElementById('btn-open-converter');
const btnTheme = document.getElementById('theme-toggle');

btnOpenConverter.onclick = () => {
  invoke('open_converter');
};

let lastKnownSettingsPath = "";
let lastKnownPort = 8000;
let lastKnownBrowser = "";

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

// --- App Logic ---

function setStatus(text) {
  elStatus.textContent = text;
  if (text.toLowerCase().includes('error')) {
    elStatus.className = 'error';
  } else {
    elStatus.className = '';
  }
}

function addLog(msg) {
  const line = document.createElement('div');
  line.className = 'log-line';
  line.textContent = msg;
  elLogs.appendChild(line);

  // Keep only last 21 lines in DOM
  while (elLogs.children.length > 21) {
    elLogs.removeChild(elLogs.firstChild);
  }

  elLogs.scrollTop = elLogs.scrollHeight;
}

async function initialLoad() {
  try {
    const status = await invoke('get_app_status');
    elSettingsPath.value = status.settings_path || "";
    elMbxcPath.textContent = status.mbxc_path || "---";
    elPortInput.value = status.port || 8000;
    elBrowserInput.value = status.browser || "";
    setStatus(status.status);
    lastKnownSettingsPath = status.settings_path;
    lastKnownPort = status.port || 8000;
    lastKnownBrowser = status.browser || "";

    // Load existing messages
    elLogs.textContent = '';
    status.messages.forEach(msg => addLog(msg));
  } catch (err) {
    console.error('Initial load failed:', err);
  }
}

// Listen for backend events
listen('backend-log', (event) => addLog(event.payload));

listen('backend-status', (event) => setStatus(event.payload));

listen('backend-config', (event) => {
  const cfg = event.payload;
  elSettingsPath.value = cfg.settings_path;
  elMbxcPath.textContent = cfg.mbxc_path;
  elPortInput.value = cfg.port;
  elBrowserInput.value = cfg.browser || "";
  lastKnownSettingsPath = cfg.settings_path;
  lastKnownPort = cfg.port;
  lastKnownBrowser = cfg.browser || "";

  // If we get messages in the config (e.g. on startup or status update), sync them
  if (cfg.messages && cfg.messages.length > 0) {
    elLogs.textContent = '';
    cfg.messages.forEach(msg => addLog(msg));
  }

  editActions.classList.add('hidden'); // Hide buttons on successful auto-config
  portEditActions.classList.add('hidden');
  portWarning.classList.add('hidden');
});

// UI Event Handlers
elSettingsPath.oninput = () => {
  const isChanged = elSettingsPath.value.trim() !== lastKnownSettingsPath;
  if (isChanged) {
    editActions.classList.remove('hidden');
  } else {
    editActions.classList.add('hidden');
  }
};

elPortInput.oninput = () => {
  const newPort = parseInt(elPortInput.value);
  const isChanged = newPort !== lastKnownPort;
  if (isChanged) {
    portEditActions.classList.remove('hidden');
    portWarning.classList.remove('hidden');
  } else {
    portEditActions.classList.add('hidden');
    portWarning.classList.add('hidden');
  }
};

elBrowserInput.oninput = () => {
  const isChanged = elBrowserInput.value.trim() !== lastKnownBrowser;
  if (isChanged) {
    browserEditActions.classList.remove('hidden');
  } else {
    browserEditActions.classList.add('hidden');
  }
};

btnApplyEdit.onclick = async () => {
  const newPath = elSettingsPath.value.trim();
  try {
    await invoke('restart_backend_with_settings', { path: newPath });
    lastKnownSettingsPath = newPath;
    editActions.classList.add('hidden');
  } catch (err) {
    alert('Apply failed: ' + err);
  }
};

btnCancelEdit.onclick = () => {
  elSettingsPath.value = lastKnownSettingsPath;
  editActions.classList.add('hidden');
};

btnApplyPort.onclick = async () => {
  const newPort = parseInt(elPortInput.value);
  if (isNaN(newPort) || newPort < 1024 || newPort > 65535) {
    alert('Invalid port number');
    return;
  }
  try {
    await invoke('update_global_port', { port: newPort });
    lastKnownPort = newPort;
    portEditActions.classList.add('hidden');
    portWarning.classList.add('hidden');
  } catch (err) {
    alert('Failed to update port: ' + err);
  }
};

btnCancelPort.onclick = () => {
  elPortInput.value = lastKnownPort;
  portEditActions.classList.add('hidden');
  portWarning.classList.add('hidden');
};

btnApplyBrowser.onclick = async () => {
  const newBrowser = elBrowserInput.value.trim() || null;
  try {
    await invoke('update_browser', { browser: newBrowser });
    lastKnownBrowser = newBrowser || "";
    browserEditActions.classList.add('hidden');
  } catch (err) {
    alert('Failed to update browser: ' + err);
  }
};

btnCancelBrowser.onclick = () => {
  elBrowserInput.value = lastKnownBrowser;
  browserEditActions.classList.add('hidden');
};

btnSelectFile.onclick = async () => {
  try {
    const selected = await invoke('select_settings_file');
    if (selected) {
      elSettingsPath.value = selected;
      await invoke('restart_backend_with_settings', { path: selected });
      lastKnownSettingsPath = selected;
      editActions.classList.add('hidden');
    }
  } catch (err) {
    alert('Selection failed: ' + err);
  }
};

btnRestart.onclick = async () => {
  try {
    await invoke('restart_backend_with_settings', { path: lastKnownSettingsPath });
    editActions.classList.add('hidden');
  } catch (err) {
    alert('Restart failed: ' + err);
  }
};

btnCopy.onclick = () => {
  const lines = Array.from(elLogs.querySelectorAll('.log-line')).map(el => el.textContent);
  navigator.clipboard.writeText(lines.join('\n')).then(() => {
    const originalText = btnCopy.textContent;
    btnCopy.textContent = 'Copied!';
    setTimeout(() => btnCopy.textContent = originalText, 2000);
  });
};

// Start Up
initTheme();
initialLoad();
