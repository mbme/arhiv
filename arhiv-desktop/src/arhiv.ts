import { createHash } from 'node:crypto';
import { execFile, spawn } from 'node:child_process';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);

const arhivBin = process.env.ARHIV_BIN ?? 'arhiv';
console.log('arhiv bin:', arhivBin);

type ServerInfo = {
  url: string;
  certificate: number[];
};

type ExtendedServerInfo = ServerInfo & {
  fingerprint: string;
};

export async function getServerInfo(): Promise<ExtendedServerInfo> {
  const result = await execFileAsync(arhivBin, ['server-info'], { encoding: 'utf8' });

  if (!result.stdout) {
    throw new Error("arhiv server-info didn't return any output");
  }

  const serverInfo = JSON.parse(result.stdout.toString()) as ServerInfo | null;
  if (!serverInfo) {
    throw new Error("arhiv server isn't running");
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
