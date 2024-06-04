import { app, Tray, Menu, nativeImage, BrowserWindow } from 'electron';
import { getServerInfo, startServer } from './arhiv';
import favicon from '../../resources/favicon-16x16.png';

// run arhiv server if not running
// add tray icon by default, with open,search,quit
// if command "open" / "search" -> show window
// do not close app if last window closed
// https://www.electronjs.org/docs/latest/api/app#apprequestsingleinstancelockadditionaldata

function showTrayIcon() {
  const icon = nativeImage.createFromDataURL(favicon);
  const tray = new Tray(icon);

  const contextMenu = Menu.buildFromTemplate([
    { label: 'Quit', type: 'normal', click: () => app.quit() },
  ]);

  tray.setToolTip('Arhiv Desktop App');
  tray.setContextMenu(contextMenu);
}

type Action = { type: 'open'; documentId?: string } | { type: 'search'; query: string };

function parseAction(args: string[]): Action {
  switch (args[0]) {
    case 'search':
      return { type: 'search', query: args[1] ?? '' };
    case 'open':
      if (!args[1]) {
        throw new Error(`Empty documentId for open action`);
      }
      return { type: 'open', documentId: args[1] };
    default:
      return { type: 'open' };
  }
}

let win: BrowserWindow;

async function handleAction(action: Action, baseUrl: string) {
  // if window already open - restore & focus
  if (win) {
    if (win.isMinimized()) {
      win.restore();
    }
    win.focus();
    console.log('Another instance already running');
    return;
  }

  win = new BrowserWindow({
    autoHideMenuBar: true,
    width: 800,
    height: 600,
  });

  await win.loadURL(baseUrl).catch(() => {
    console.error('failed to open Arhiv');
  });

  switch (action.type) {
    case 'search': {
      win.webContents.send('search', action.query);
      break;
    }
    case 'open': {
      if (action.documentId) {
        win.webContents.send('open', action.documentId);
      }
      break;
    }
  }
}

async function start(args: string[]) {
  console.log('args:', ...args);

  const action = parseAction(args);

  if (!app.requestSingleInstanceLock(action)) {
    app.quit();
    return;
  }

  if (args.includes('--start-server')) {
    startServer(() => {
      app.quit();
    });
  }

  const serverInfo = await getServerInfo();
  console.log('server base url:', serverInfo.url);

  app.on('second-instance', (_event, _commandLine, _workingDirectory, additionalData) => {
    const actionFromSecondInstance = additionalData as Action;
    console.log('Got action from second instance:', actionFromSecondInstance);

    handleAction(actionFromSecondInstance, serverInfo.url).catch((e) => {
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

  await app.whenReady();

  showTrayIcon();

  await handleAction(action, serverInfo.url);
}

const args = process.argv.slice(2);
start(args).catch((e) => {
  console.error('Failed to start', e);
  app.quit();
});
