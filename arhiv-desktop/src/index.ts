import { app, Tray, Menu, nativeImage, BrowserWindow } from 'electron';
import { getServerInfo, startServer, waitForServer } from './arhiv';
import favicon from '../../resources/favicon-16x16.png';

function showTrayIcon(uiUrl: string) {
  const icon = nativeImage.createFromDataURL(favicon);
  const tray = new Tray(icon);

  const contextMenu = Menu.buildFromTemplate([
    { label: 'Open', type: 'normal', click: () => void handleAction({ type: 'open' }, uiUrl) },
    {
      label: 'Search',
      type: 'normal',
      click: () => void handleAction({ type: 'search', query: '' }, uiUrl),
    },
    { type: 'separator' },
    { label: 'Quit', type: 'normal', click: () => app.quit() },
  ]);

  tray.setToolTip('Arhiv Desktop App');
  tray.setContextMenu(contextMenu);
}

export type Action = { type: 'open'; documentId: string } | { type: 'search'; query: string };

function parseAction(args: string[]): Action | undefined {
  const searchArg = args.find((item) => item.startsWith('search='));
  if (searchArg) {
    const query = searchArg.substring('search='.length);
    return { type: 'search', query };
  }

  const openArg = args.find((item) => item.match(/^open=?/));
  if (openArg) {
    const documentId = openArg.substring('open='.length);

    return { type: 'open', documentId };
  }
}

let win: BrowserWindow | undefined;

async function handleAction(action: Action, uiUrl: string) {
  console.log('Handling action', action);

  // if window already open - restore & focus
  if (win) {
    console.log('Window already open, reusing');
    if (win.isMinimized()) {
      win.restore();
    }
    win.focus();
  } else {
    console.log('Opening new window');
    win = new BrowserWindow({
      autoHideMenuBar: true,
      width: 800,
      height: 600,
    });

    win.on('closed', () => {
      win = undefined;
    });

    await win.loadURL(uiUrl).catch(() => {
      console.error('failed to open Arhiv');
    });
  }

  switch (action.type) {
    case 'search': {
      await win.webContents.executeJavaScript(
        `window.WORKSPACE.showSearchDialog(${JSON.stringify(action.query)})`,
      );
      break;
    }
    case 'open': {
      if (action.documentId) {
        await win.webContents.executeJavaScript(
          `window.WORKSPACE.openDocument(${JSON.stringify(action.documentId)}, true)`,
        );
      }
      break;
    }
    default: {
      throw new Error('Unhandled action');
    }
  }
}

async function start(args: string[]) {
  console.log('args:', ...args);

  const action = parseAction(args);
  console.log('action:', action);

  if (!app.requestSingleInstanceLock(action)) {
    app.quit();
    return;
  }

  if (args.includes('--start-server')) {
    startServer(() => {
      app.quit();
    });

    await waitForServer();
  }

  const serverInfo = await getServerInfo();
  if (!serverInfo) {
    throw new Error("arhiv server isn't running");
  }
  console.log('server base url:', serverInfo.uiUrl);

  app.on('second-instance', (_event, _commandLine, _workingDirectory, additionalData) => {
    const actionFromSecondInstance = additionalData as Action;
    console.log('Got action from second instance:', actionFromSecondInstance);

    handleAction(actionFromSecondInstance, serverInfo.uiUrl).catch((e) => {
      console.error('Action from second instance failed', e);
    });
  });

  // SSL/TSL: self signed certificate support
  app.on('certificate-error', (event, _webContents, _url, _error, certificate, callback) => {
    const isValidHash = certificate.fingerprint === serverInfo.fingerprint;
    if (isValidHash) {
      // disable default behaviour (stop loading the page)
      event.preventDefault();

      // accept certificate
      callback(true);
    } else {
      console.error('Invalid certificate fingerprint', certificate.fingerprint);
      console.error('Expected fingerprint', serverInfo.fingerprint);
    }
  });

  // needed to prevent quiting the app when last window is closed
  app.on('window-all-closed', () => {
    console.log('last window closed');
  });

  await app.whenReady();

  showTrayIcon(serverInfo.uiUrl);

  if (action) {
    await handleAction(action, serverInfo.uiUrl);
  }
}

const args = process.argv.slice(2);
start(args).catch((e) => {
  console.error('Failed to start', e);
  app.quit();
});
