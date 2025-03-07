import { PhysicalSize, PhysicalPosition } from '@tauri-apps/api/dpi';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { currentMonitor } from '@tauri-apps/api/window';

const DEFAULT_WINDOW_WIDTH = 900;
const DEFAULT_WINDOW_HEIGHT = 150;

// Position 1/4 of the way down the screen
export async function setIntialPosition() {
  const window = WebviewWindow.getCurrent();

  await window.setSize(new PhysicalSize(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT));
  await window.center();

  const currentPosition = await window.innerPosition();
  const monitor = await currentMonitor();
  const screenHeight = monitor!.size.height;

  const xNew = currentPosition.x;
  const yNew = Math.round(screenHeight * 0.25);

  console.info(`Setting position to [${xNew}, ${yNew}]`);

  await window.setPosition(new PhysicalPosition(xNew, yNew));
}

export async function scaleWindow(numSuggestions: number) {
  const window = WebviewWindow.getCurrent();
  const currentSize = await window.size();
  const newHeight = DEFAULT_WINDOW_HEIGHT + numSuggestions * 50;
  await window.setSize(new PhysicalSize(currentSize.width, newHeight));
}
