import { spawn, ChildProcess } from 'child_process';
import { join } from 'path';
import { platform } from 'os';

export default async function globalSetup() {
  const tauriProcess = spawn(
    process.platform === 'win32' ? 'npx.cmd' : 'npx',
    ['tauri', 'dev'],
    {
      cwd: join(process.cwd()),
      env: { ...process.env, CI: 'e2e' },
      stdio: 'inherit',
    }
  );

  (global as any).__TAURI_PROCESS__ = tauriProcess;

  // Wait for Tauri app to start
  await new Promise<void>((resolve) => {
    const checkReady = setInterval(() => {
      // Tauri dev mode typically runs on a random port or uses webview
      // For E2E, we'll use the webview directly
      resolve();
    }, 5000);
    setTimeout(() => {
      clearInterval(checkReady);
      resolve();
    }, 30000);
  });
}
