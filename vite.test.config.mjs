import { defineConfig } from 'vite';
import { resolve } from 'path';

/**
 * Tauri APIをモックするViteプラグイン
 * フロントエンドのE2Eテスト用にwindow.__TAURI__を注入
 */
function tauriApiMockPlugin() {
  const mockData = {
    settings: {
      selected_engine: 'DeepL',
      ui_language: 'ja',
      default_source_lang: 'en',
      default_target_lang: 'ja',
      api_keys: {},
    },
    localizedStrings: {
      AppTitle: 'Factorio Mod Translator',
      SelectMod: 'Modを選択',
      Translate: '翻訳',
      Glossary: '用語集',
      History: '履歴',
      Settings: '設定',
      ApiConfigured: '設定済み',
      ApiNotConfigured: '未設定',
    },
    mod: {
      name: 'test_mod',
      title: 'Test Mod',
      version: '1.0.0',
      locale_files: [{
        file_path: 'locale/en/test.cfg',
        language_code: 'en',
        entries: [
          { section: 'entity-name', key: 'iron-ore', value: 'Iron Ore' },
          { section: 'item-name', key: 'copper-plate', value: 'Copper Plate' },
          { section: 'technology-name', key: 'steel-processing', value: 'Steel Processing' },
        ],
      }],
    },
    glossary: [],
    history: [],
  };

  return {
    name: 'tauri-api-mock',
    transformIndexHtml(html) {
      const mockScript = `
        <script type="module">
          window.__TAURI_MOCK_DATA__ = ${JSON.stringify(mockData)};

          window.__TAURI__ = {
            core: {
              invoke: async (cmd, args) => {
                console.log('[TauriMock] invoke:', cmd, args);
                return handleInvoke(cmd, args);
              }
            },
            event: {
              listen: async (event, handler) => {
                console.log('[TauriMock] listen:', event);
                return () => {};
              },
              emit: async (event, payload) => {
                console.log('[TauriMock] emit:', event, payload);
              }
            }
          };

          async function handleInvoke(cmd, args) {
            const mock = window.__TAURI_MOCK_DATA__;

            switch (cmd) {
              case 'get_settings':
                return mock.settings;
              case 'save_settings':
                Object.assign(mock.settings, args.settings);
                return null;
              case 'get_localized_strings':
                return mock.localizedStrings;
              case 'select_mod_path':
              case 'select_mod_zip_path':
                return '/mock/mod/path';
              case 'load_mod':
                return mock.mod;
              case 'translate_mod':
                return mock.mod.locale_files[0].entries.map(e => ({
                  section: e.section,
                  key: e.key,
                  source_text: e.value,
                  translated_text: '[翻訳] ' + e.value,
                  source: 'Mock',
                  is_edited: false,
                }));
              case 'save_translation':
                return null;
              case 'get_glossary':
                return mock.glossary;
              case 'add_glossary_entry':
                mock.glossary.push(args.entry);
                return null;
              case 'remove_glossary_entry':
                mock.glossary = mock.glossary.filter(e => e.source_term !== args.term);
                return null;
              case 'get_history':
                return mock.history;
              case 'save_api_key':
                mock.settings.api_keys[args.engine] = '***';
                return null;
              case 'log_info':
              case 'log_warn':
              case 'log_error':
                return null;
              default:
                console.warn('[TauriMock] Unknown command:', cmd);
                return null;
            }
          }
        </script>
      `;
      return html.replace('</head>', `${mockScript}</head>`);
    },
  };
}

export default defineConfig({
  root: resolve(__dirname, 'src'),
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    outDir: resolve(__dirname, 'dist'),
  },
  plugins: [tauriApiMockPlugin()],
});
