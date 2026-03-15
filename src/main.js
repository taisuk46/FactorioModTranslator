const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const log = (level, message) => {
  const payload = typeof message === 'string' ? message : JSON.stringify(message);
  invoke(`log_${level}`, { message: payload });
};
const info = (message) => log('info', message);
const warn = (message) => log('warn', message);
const error = (message) => log('error', message);

let currentMod = null;
let currentSettings = null;
let localizedStrings = {};
let lastResults = [];

async function init() {
  await info("Application frontend initializing...");
  // Setup Tab switching first (independent of Backend)
  const tabs = document.querySelectorAll('.tab');
  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      const viewId = tab.getAttribute('data-view');
      switchView(viewId);
    });
  });

  try {
    // Load settings
    currentSettings = await invoke('get_settings');
    
    // Apply initial localization from Rust
    await applyLocalization(currentSettings.ui_language || 'ja');

    // Initial load of content
    loadGlossary();
    loadHistory();
    populateSettings();
  } catch (e) {
    await warn(`Backend not available (Tauri check): ${e}`);
    // Fallback settings for UI preview
    currentSettings = { selected_engine: 'DeepL', ui_language: 'ja' };
  }

  // Mod Selection
  const btnBrowse = document.getElementById('btn-browse');
  if (btnBrowse) {
    btnBrowse.addEventListener('click', async () => {
      const path = prompt(localizedStrings.PromptEnterModPath || "Enter Mod Folder or Zip path:");
      if (path) {
        try {
          currentMod = await invoke('load_mod', { path });
          await info({ event: "mod_loaded_ui", title: currentMod.title, version: currentMod.version });
          showStatus(`Loaded: ${currentMod.title} (${currentMod.version})`);
          switchView('translation-preview');
          renderPreview();
        } catch (e) {
          await error(`Error loading mod: ${e}`);
          showError("Error loading mod: " + e);
        }
      }
    });
  }

  // Translation
  const btnTranslate = document.getElementById('btn-translate');
  if (btnTranslate) {
    btnTranslate.addEventListener('click', async () => {
      if (!currentMod) return;
      try {
        showStatus("Translation started...");
        const results = await invoke('translate_mod', {
          modInfo: currentMod,
          mode: 'NewTranslation',
          sourceLang: 'en',
          targetLang: 'ja',
          engineType: currentSettings.selected_engine
        });
        lastResults = results;
        renderResults(results);
        showStatus("Translation completed!");
      } catch (e) {
        await error(`Translation failed: ${e}`);
        showError("Translation failed: " + e);
      }
    });
  }

  // Save Mod
  const btnSaveMod = document.getElementById('btn-save-mod');
  if (btnSaveMod) {
    btnSaveMod.addEventListener('click', async () => {
      if (!currentMod || lastResults.length === 0) {
        showError("No translation to save.");
        return;
      }

      try {
        showStatus("Saving mod...");
        
        // Update lastResults with current input values from UI
        const inputs = document.querySelectorAll('#translation-list .row-target input');
        inputs.forEach((input, index) => {
          if (lastResults[index]) {
            lastResults[index].translated_text = input.value;
          }
        });

        await invoke('save_translation', {
          modInfo: currentMod,
          translations: lastResults,
          targetLang: 'ja'
        });
        
        await info({ event: "mod_saved_ui", mod: currentMod.name });
        showStatus("Mod saved successfully!");
      } catch (e) {
        await error(`Save failed: ${e}`);
        showError("Save failed: " + e);
      }
    });
  }
}

async function applyLocalization(lang) {
  try {
    localizedStrings = await invoke('get_localized_strings', { lang });
    
    document.getElementById('app-title').innerText = localizedStrings.AppTitle;
    document.getElementById('tab-mod-selection').innerText = localizedStrings.SelectMod;
    document.getElementById('tab-translation-preview').innerText = localizedStrings.Translate;
    document.getElementById('tab-glossary').innerText = localizedStrings.Glossary;
    document.getElementById('tab-history').innerText = localizedStrings.History;
    document.getElementById('tab-settings').innerText = localizedStrings.Settings;
    
    const titleMod = document.getElementById('title-mod-selection');
    if (titleMod) titleMod.innerText = localizedStrings.SelectMod;
    
    const labelDrop = document.getElementById('label-drop-mod');
    if (labelDrop) labelDrop.innerText = localizedStrings.SelectMod;
    
    const btnBrowse = document.getElementById('btn-browse');
    if (btnBrowse) btnBrowse.innerText = localizedStrings.SelectMod;

    // ... more labels ...
  } catch (e) {
    await error(`Localization failed: ${e}`);
  }
}

function showStatus(msg) {
  document.getElementById('status-bar').innerText = msg;
}

function showError(msg) {
  const bar = document.getElementById('status-bar');
  bar.innerText = msg;
  bar.style.color = '#f44336';
  setTimeout(() => { bar.style.color = ''; }, 5000);
}

function switchView(viewId) {
  document.querySelectorAll('.view').forEach(v => v.classList.remove('active'));
  document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
  
  document.getElementById(viewId).classList.add('active');
  const tab = document.querySelector(`.tab[data-view="${viewId}"]`);
  if (tab) tab.classList.add('active');

  if (viewId === 'glossary') loadGlossary();
  if (viewId === 'history') loadHistory();
}

function renderPreview() {
  const list = document.getElementById('translation-list');
  list.innerHTML = '';
  if (!currentMod) return;

  lastResults = [];
  currentMod.locale_files.forEach(file => {
    const fileHeader = document.createElement('h3');
    fileHeader.innerText = file.file_path;
    fileHeader.className = 'file-header';
    list.appendChild(fileHeader);

    const table = document.createElement('table');
    table.className = 'data-table';
    table.innerHTML = `
      <thead>
        <tr>
          <th style="width: 25%">Key</th>
          <th style="width: 35%">Source Text</th>
          <th style="width: 40%">Translation</th>
        </tr>
      </thead>
      <tbody></tbody>
    `;
    const tbody = table.querySelector('tbody');

    file.entries.forEach(entry => {
      // Add to lastResults so it can be saved even if not auto-translated
      lastResults.push({
        section: entry.section,
        key: entry.key,
        source_text: entry.value,
        translated_text: entry.value, // Default to original for manual edit
        source: 'Manual',
        is_edited: false
      });

      const row = document.createElement('tr');
      row.innerHTML = `
        <td class="row-key">${entry.section} > ${entry.key}</td>
        <td class="row-source">${entry.value}</td>
        <td class="row-target"><input type="text" class="form-group" style="margin:0; width:100%" value="${entry.value}"></td>
      `;
      tbody.appendChild(row);
    });
    list.appendChild(table);
  });
}

function renderResults(results) {
  const list = document.getElementById('translation-list');
  list.innerHTML = '<div class="panel-header"><h3 class="panel-title">Translation Results</h3></div>';
  
  const table = document.createElement('table');
  table.className = 'data-table';
  table.innerHTML = `
    <thead>
      <tr>
        <th style="width: 20%">Key</th>
        <th style="width: 30%">Source</th>
        <th style="width: 40%">Translation</th>
        <th style="width: 10%">Type</th>
      </tr>
    </thead>
    <tbody></tbody>
  `;
  const tbody = table.querySelector('tbody');

  results.forEach(res => {
    const row = document.createElement('tr');
    const badgeClass = `badge-${res.source.toLowerCase().includes('vanilla') ? 'vanilla' : (res.source.toLowerCase().includes('api') ? 'api' : 'history')}`;
    row.innerHTML = `
      <td class="row-key">${res.section}.${res.key}</td>
      <td class="row-source">${res.source_text}</td>
      <td class="row-target"><input type="text" class="form-group" style="margin:0; width:100%" value="${res.translated_text}"></td>
      <td><span class="badge ${badgeClass}">${res.source}</span></td>
    `;
    tbody.appendChild(row);
  });
  list.appendChild(table);
}

async function loadGlossary() {
  const container = document.getElementById('glossary-list');
  try {
    const entries = await invoke('get_glossary');
    container.innerHTML = `
      <table class="data-table">
        <thead>
          <tr>
            <th>Source Term</th>
            <th>Target Term</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          ${entries.map(e => `
            <tr>
              <td>${e.source_term}</td>
              <td>${e.target_term}</td>
              <td><button class="btn btn-danger btn-sm" onclick="deleteGlossaryEntry('${e.source_term}')">Delete</button></td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    `;
  } catch (e) {
    await error(`Failed to load glossary: ${e}`);
    showError("Failed to load glossary");
  }
}

window.deleteGlossaryEntry = async (term) => {
  if (confirm(`Delete '${term}'?`)) {
    await invoke('remove_glossary_entry', { term });
    loadGlossary();
  }
};

document.getElementById('btn-add-glossary').addEventListener('click', async () => {
  const source = prompt("Source term:");
  const target = prompt("Target term:");
  if (source && target) {
    const entry = {
      source_term: source,
      target_term: target,
      source_lang: "en",
      target_lang: "ja",
      exclude_from_translation: false
    };
    await invoke('add_glossary_entry', { entry });
    loadGlossary();
  }
});

async function loadHistory() {
  const container = document.getElementById('history-list');
  try {
    const history = await invoke('get_history');
    container.innerHTML = `
      <table class="data-table">
        <thead>
          <tr>
            <th>Mod</th>
            <th>Key</th>
            <th>Source</th>
            <th>Translation</th>
          </tr>
        </thead>
        <tbody>
          ${history.map(h => `
            <tr>
              <td style="font-weight:bold">${h.mod_name}</td>
              <td class="row-key">${h.section}.${h.key}</td>
              <td>${h.source_text}</td>
              <td>${h.translated_text}</td>
            </tr>
          `).join('')}
        </tbody>
      </table>
    `;
  } catch (e) {
    await error(`Failed to load history: ${e}`);
    showError("Failed to load history");
  }
}

async function populateSettings() {
  const engineSelect = document.getElementById('engine-select');
  if (engineSelect && currentSettings) {
    engineSelect.value = currentSettings.selected_engine;
  }
}

document.getElementById('btn-save-key').addEventListener('click', async () => {
  const engineSelect = document.getElementById('engine-select');
  const apiKeyInput = document.getElementById('api-key-input');
  
  const selectedEngine = engineSelect.value;
  const apiKey = apiKeyInput.value.trim();

  try {
    // 1. Save engine type in AppSettings
    currentSettings.selected_engine = selectedEngine;
    await invoke('save_settings', { settings: currentSettings });

    // 2. Save API key securely if provided
    if (apiKey) {
      const engineName = selectedEngine === 'DeepL' ? 'DeepL' : 'Google';
      await invoke('save_api_key', { engine: engineName, key: apiKey });
      apiKeyInput.value = ''; // Clear for security
    }

    showStatus("Settings saved successfully!");
  } catch (e) {
    await error(`Failed to save settings: ${e}`);
    showError("Failed to save settings: " + e);
  }
});

window.addEventListener('DOMContentLoaded', init);
