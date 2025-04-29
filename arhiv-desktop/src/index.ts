import { app, BrowserWindow, session, Notification, shell } from 'electron';
import { ExtendedServerInfo, startServer } from './arhiv';

export type Action = { type: 'open'; documentId?: string } | { type: 'search'; query: string };

const DEFAULT_ACTION: Action = { type: 'open' };

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

async function handleAction(action: Action, serverInfo: ExtendedServerInfo) {
  console.log('Handling action', action);

  // if window already open - restore & focus
  if (win) {
    console.log('Window already open, reusing');
    if (win.isMinimized()) {
      win.restore();
    }
    win.flashFrame(true);
    win.show();
    win.focus();
    new Notification({
      title: 'Arhiv Desktop',
      body: 'The other instance already running',
      urgency: 'critical',
    }).show();
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

    // open external links in default system browser instead of a new electron window
    win.webContents.setWindowOpenHandler((details) => {
      void shell.openExternal(details.url);

      return { action: 'deny' };
    });

    await session.defaultSession.cookies.set({
      url: serverInfo.uiUrl,
      name: 'AuthToken',
      value: serverInfo.authToken,
      secure: true,
      sameSite: 'strict',
    });

    await win.loadURL(serverInfo.uiUrl).catch(() => {
      console.error('failed to open Arhiv');
    });
  }

  switch (action.type) {
    case 'search': {
      await win.webContents.executeJavaScript(
        `window.APP.workspace.showSearchDialog(${JSON.stringify(action.query)})`,
      );
      break;
    }
    case 'open': {
      if (action.documentId) {
        await win.webContents.executeJavaScript(
          `window.APP.workspace.openDocument(${JSON.stringify(action.documentId)}, true)`,
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

  const verbosityArg = args.find((item) => /^-v+$/.test(item));

  if (!app.requestSingleInstanceLock(action)) {
    console.log('The other instance already running, quiting');
    app.quit();
    return;
  }

  let serverInfo: ExtendedServerInfo;
  try {
    const arhivArgs = [];
    if (verbosityArg) {
      arhivArgs.push(verbosityArg);
    }
    serverInfo = await startServer(arhivArgs);
  } catch (e) {
    console.error('Failed to start server', e);
    app.quit();
    return;
  }

  console.log('Arhiv server base url:', serverInfo.uiUrl);

  app.on('second-instance', (_event, _commandLine, _workingDirectory, additionalData) => {
    const actionFromSecondInstance = additionalData as Action | undefined;
    console.log('Got action from second instance:', actionFromSecondInstance);

    handleAction(actionFromSecondInstance ?? DEFAULT_ACTION, serverInfo).catch((e: unknown) => {
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
    app.quit();
  });

  await app.whenReady();

  await handleAction(action ?? DEFAULT_ACTION, serverInfo);
}

const args = process.argv.slice(2);
start(args).catch((e: unknown) => {
  console.error('Failed to start', e);
  app.quit();
});
