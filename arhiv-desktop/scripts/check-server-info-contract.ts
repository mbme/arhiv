import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

/**
 * Purpose:
 * Guard the desktop startup contract for server bootstrapping.
 *
 * How:
 * 1) Verify the CLI (`binutils/src/bin/arhiv.rs`) emits the exact
 *    `@@SERVER_INFO:` marker before JSON payload.
 * 2) Verify desktop parser (`src/arhiv.ts`) looks for the same marker.
 * 3) Verify desktop parser slices payload using marker length.
 * Any mismatch means desktop can fail to discover server metadata at startup.
 */
const repoRoot = path.resolve(process.cwd(), '..');
const rustCliPath = path.join(repoRoot, 'binutils', 'src', 'bin', 'arhiv.rs');
const desktopPath = path.join(process.cwd(), 'src', 'arhiv.ts');

const rustCli = fs.readFileSync(rustCliPath, 'utf8');
const desktop = fs.readFileSync(desktopPath, 'utf8');

const marker = '@@SERVER_INFO:';

const rustHasMarker = rustCli.includes(`"${marker} {}"`);
if (!rustHasMarker) {
  console.error(
    `Missing expected marker emission in ${path.relative(repoRoot, rustCliPath)}`,
  );
  process.exit(1);
}

const desktopReadsMarker = desktop.includes(`line.startsWith('${marker}')`);
if (!desktopReadsMarker) {
  console.error(
    `Missing expected marker parser in ${path.relative(process.cwd(), desktopPath)}`,
  );
  process.exit(1);
}

const desktopSlicesMarker = desktop.includes(`line.slice('${marker}'.length)`);
if (!desktopSlicesMarker) {
  console.error(
    `Desktop parser no longer slices payload using marker length in ${path.relative(process.cwd(), desktopPath)}`,
  );
  process.exit(1);
}

console.log(`Server info marker contract OK: ${marker}`);
