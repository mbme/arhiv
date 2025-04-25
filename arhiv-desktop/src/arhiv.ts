import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { Readable, PassThrough } from 'node:stream';
import { createInterface } from 'node:readline';

function getArhivBin(): string {
  const isProduction = process.env.NODE_ENV === 'production';
  if (isProduction) {
    return 'arhiv';
  }

  const arhivBin = process.env.ARHIV_BIN;
  if (!arhivBin) {
    throw new Error('ARHIV_BIN must be specified in development mode');
  }

  console.log('Arhiv bin:', arhivBin);

  return arhivBin;
}

type ServerInfo = {
  uiUrl: string;
  healthUrl: string;
  certificate: number[];
  authToken: string;
};

export type ExtendedServerInfo = ServerInfo & {
  fingerprint: string;
};

async function readServerInfo(input: Readable): Promise<ExtendedServerInfo> {
  const rl = createInterface({ input });

  for await (const line of rl) {
    if (line.startsWith('@@SERVER_INFO:')) {
      const json = line.slice('@@SERVER_INFO:'.length);

      const serverInfo = JSON.parse(json) as ServerInfo | null;
      if (!serverInfo) {
        throw new Error('Failed to parse server info after marker');
      }

      return {
        ...serverInfo,
        fingerprint: getCertificateFingerprint(serverInfo.certificate),
      };
    }
  }

  throw new Error('No server info marker found');
}

function getCertificateFingerprint(certificate: number[]): string {
  const hash = createHash('sha256');
  hash.update(Buffer.from(certificate));
  const base64Hash = hash.digest('base64');

  return `sha256/${base64Hash}`;
}

export async function startServer(): Promise<ExtendedServerInfo> {
  console.log('Starting Arhiv server');

  const result = spawn(getArhivBin(), ['server', '--json', '-v'], {
    stdio: ['ignore', 'inherit', 'pipe'],
  });

  result.on('close', (code) => {
    console.log(`Arhiv server: Process exited with code ${code}`);
  });

  process.on('exit', () => {
    result.kill();
  });

  const tap = new PassThrough();
  result.stderr.pipe(tap);
  tap.pipe(process.stderr);

  return await readServerInfo(tap);
}
