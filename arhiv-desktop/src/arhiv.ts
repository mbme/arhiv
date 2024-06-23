import https from 'node:https';
import { createHash } from 'node:crypto';
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);

const arhivBin = process.env.ARHIV_BIN ?? 'arhiv';
console.log('arhiv bin:', arhivBin);

type ServerInfo = {
  uiUrl: string;
  healthUrl: string;
  certificate: number[];
};

type ExtendedServerInfo = ServerInfo & {
  fingerprint: string;
};

export async function getServerInfo(): Promise<ExtendedServerInfo | undefined> {
  const result = await execFileAsync(arhivBin, ['server-info'], { encoding: 'utf8' });

  if (!result.stdout) {
    throw new Error("arhiv server-info didn't return any output");
  }

  const serverInfo = JSON.parse(result.stdout.toString()) as ServerInfo | null;
  if (!serverInfo) {
    return undefined;
  }

  return {
    ...serverInfo,
    fingerprint: getCertificateFingerprint(serverInfo.certificate),
  };
}

function getCertificateFingerprint(certificate: number[]): string {
  const hash = createHash('sha256');
  hash.update(Buffer.from(certificate));
  const base64Hash = hash.digest('base64');

  return `sha256/${base64Hash}`;
}

export function startServer(onError: () => void): void {
  console.log('starting arhiv server');
  const result = spawn(arhivBin, ['server'], { stdio: 'inherit' });

  // TODO wait for server on port

  result.on('close', (code) => {
    console.log(`Arhiv server: Process exited with code ${code}`);
  });

  result.on('error', (err) => {
    console.error('Arhiv server: Failed to start process:', err);
    onError();
  });

  process.on('exit', () => {
    result.kill();
  });
}

async function isServerRunning(url: string) {
  return new Promise((resolve) => {
    const req = https.request(
      url,
      {
        rejectUnauthorized: false,
      },
      (res) => {
        if (res.statusCode === 200) {
          resolve(true);
        } else {
          resolve(false);
        }
      },
    );

    req.on('error', () => {
      resolve(false);
    });

    req.end();
  });
}

function promiseTimeout(timeoutMs: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, timeoutMs));
}

export async function waitForServer(): Promise<void> {
  const MAX_ATTEMPTS = 20;

  for (let i = 0; i < MAX_ATTEMPTS; i += 1) {
    const serverInfo = await getServerInfo();
    if (serverInfo) {
      console.log('waitForServer: checking server url %s', serverInfo.healthUrl);
      const running = await isServerRunning(serverInfo.healthUrl);

      if (running) {
        console.log('waitForServer: Server is running');
        return;
      }
    }

    console.log('waitForServer: attempt %s failed, waiting', i);

    await promiseTimeout(150);
  }

  throw new Error('Waiting for server timed out');
}
