import { createHash } from 'node:crypto';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';

type ServerInfo = {
  url: string;
  certificate: number[];
};

type ExtendedServerInfo = ServerInfo & {
  fingerprint: string;
};

export async function getServerInfo(arhivBin: string): Promise<ExtendedServerInfo> {
  const result = await promisify(execFile)(arhivBin, ['server-info'], { encoding: 'utf8' });

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
