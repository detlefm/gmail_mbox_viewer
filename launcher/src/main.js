const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const elPortInput = document.getElementById('port-input');
const elBrowserInput = document.getElementById('browser-input');
const elLogs = document.getElementById('log-container');
const globalActions = document.getElementById('global-actions');
const portWarning = document.getElementById('port-warning');

const btnApplyAll = document.getElementById('btn-apply-all');
const btnCancelAll = document.getElementById('btn-cancel-all');
const btnCopy = document.getElementById('btn-copy-logs');

let lastKnownSettingsPath = "";
let lastKnownPort = 8000;
let lastKnownBrowser = "";

// --- App Logic ---

function addLog(msg) {
  const line = document.createElement('div');
  line.className = 'log-line';
  line.textContent = msg;
  elLogs.appendChild(line);

  while (elLogs.children.length > 21) {
    elLogs.removeChild(elLogs.firstChild);
  }

  elLogs.scrollTop = elLogs.scrollHeight;
}

async function initialLoad() {
  try {
    const status = await invoke('get_app_status');
    elPortInput.value = status.port || 8000;
    elBrowserInput.value = status.browser || "";
    lastKnownSettingsPath = status.settings_path;
    lastKnownPort = status.port || 8000;
    lastKnownBrowser = status.browser || "";

    status.messages.forEach(msg => addLog(msg));
  } catch (err) {
    console.error('Initial load failed:', err);
  }
}

listen('backend-log', (event) => addLog(event.payload));

listen('backend-config', (event) => {
  const cfg = event.payload;
  elPortInput.value = cfg.port;
  elBrowserInput.value = cfg.browser || "";
  lastKnownSettingsPath = cfg.settings_path;
  lastKnownPort = cfg.port;
  lastKnownBrowser = cfg.browser || "";

  if (cfg.messages && cfg.messages.length > 0) {
    elLogs.textContent = '';
    cfg.messages.forEach(msg => addLog(msg));
  }

  checkForChanges();
});

function checkForChanges() {
  const newPort = parseInt(elPortInput.value);
  const newBrowser = elBrowserInput.value.trim();

  const portChanged = !isNaN(newPort) && newPort !== lastKnownPort;
  const browserChanged = newBrowser !== lastKnownBrowser;

  if (portChanged || browserChanged) {
    globalActions.classList.remove('hidden');
    if (portChanged) portWarning.classList.remove('hidden');
    else portWarning.classList.add('hidden');
  } else {
    globalActions.classList.add('hidden');
    portWarning.classList.add('hidden');
  }
}

elPortInput.oninput = checkForChanges;
elBrowserInput.oninput = checkForChanges;

btnApplyAll.onclick = async () => {
  const newPort = parseInt(elPortInput.value);
  const newBrowser = elBrowserInput.value.trim() || null;

  if (isNaN(newPort) || newPort < 1024 || newPort > 65535) {
    alert('Invalid port number');
    return;
  }

  try {
    if (newPort !== lastKnownPort) {
      await invoke('update_global_port', { port: newPort });
      lastKnownPort = newPort;
    }
    if (newBrowser !== (lastKnownBrowser || null)) {
      await invoke('update_browser', { browser: newBrowser });
      lastKnownBrowser = newBrowser || "";
    }
    globalActions.classList.add('hidden');
    portWarning.classList.add('hidden');
  } catch (err) {
    alert('Failed to update settings: ' + err);
  }
};

btnCancelAll.onclick = () => {
  elPortInput.value = lastKnownPort;
  elBrowserInput.value = lastKnownBrowser;
  globalActions.classList.add('hidden');
  portWarning.classList.add('hidden');
};

btnCopy.onclick = () => {
  const lines = Array.from(elLogs.querySelectorAll('.log-line')).map(el => el.textContent);
  navigator.clipboard.writeText(lines.join('\n')).then(() => {
    const originalText = btnCopy.textContent;
    btnCopy.textContent = 'Copied!';
    setTimeout(() => btnCopy.textContent = originalText, 2000);
  });
};

initialLoad();
