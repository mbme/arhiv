import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

/**
 * Purpose:
 * Detect API contract drift between Rust DTOs and TypeScript DTOs.
 *
 * How:
 * 1) Parse `APIRequest` and `APIResponse` variant names from Rust
 *    (`src/ui/dto.rs`) by reading enum variant identifiers.
 * 2) Parse the same variant names from TypeScript (`src/ui/dto.ts`) by reading
 *    discriminated union `typeName` values.
 * 3) Compare both sets and fail with an explicit diff if either side is missing
 *    variants.
 */
const workspaceRoot = process.cwd();

const dtoRustPath = path.join(workspaceRoot, 'src', 'ui', 'dto.rs');
const dtoTsPath = path.join(workspaceRoot, 'src', 'ui', 'dto.ts');

const dtoRust = fs.readFileSync(dtoRustPath, 'utf8');
const dtoTs = fs.readFileSync(dtoTsPath, 'utf8');

function extractRustEnumBlock(source: string, enumName: string): string {
  const enumStart = source.indexOf(`pub enum ${enumName}`);
  if (enumStart < 0) {
    throw new Error(`Missing Rust enum: ${enumName}`);
  }

  const braceStart = source.indexOf('{', enumStart);
  if (braceStart < 0) {
    throw new Error(`Missing Rust enum body start: ${enumName}`);
  }

  let depth = 1;
  for (let i = braceStart + 1; i < source.length; i += 1) {
    const char = source[i];
    if (char === '{') {
      depth += 1;
    } else if (char === '}') {
      depth -= 1;
      if (depth === 0) {
        return source.slice(braceStart + 1, i);
      }
    }
  }

  throw new Error(`Missing Rust enum body end: ${enumName}`);
}

function extractRustVariants(source: string, enumName: string): Set<string> {
  const block = extractRustEnumBlock(source, enumName);
  const variants = new Set<string>();

  for (const line of block.split('\n')) {
    const match = line.match(/^\s*([A-Za-z][A-Za-z0-9_]*)\s*(?:\{|,)/);
    if (match) {
      const variant = match[1];
      if (variant) {
        variants.add(variant);
      }
    }
  }

  return variants;
}

function extractTsTypeBlock(
  source: string,
  typeName: string,
  nextTypeName: string,
): string {
  const start = source.indexOf(`export type ${typeName} =`);
  if (start < 0) {
    throw new Error(`Missing TypeScript type: ${typeName}`);
  }

  const end = source.indexOf(`export type ${nextTypeName} =`, start + 1);
  if (end < 0) {
    throw new Error(
      `Missing TypeScript type boundary: ${typeName} -> ${nextTypeName}`,
    );
  }

  return source.slice(start, end);
}

function extractTsTypeNames(
  source: string,
  typeName: string,
  nextTypeName: string,
): Set<string> {
  const block = extractTsTypeBlock(source, typeName, nextTypeName);
  const variants = new Set<string>();
  const regex = /typeName:\s*'([^']+)'/g;

  for (let match = regex.exec(block); match; match = regex.exec(block)) {
    const variant = match[1];
    if (variant) {
      variants.add(variant);
    }
  }

  return variants;
}

function diff(left: Set<string>, right: Set<string>): string[] {
  return [...left].filter((item) => !right.has(item)).sort();
}

function assertEqualSet(label: string, left: Set<string>, right: Set<string>) {
  const missingInRight = diff(left, right);
  const missingInLeft = diff(right, left);

  if (missingInLeft.length === 0 && missingInRight.length === 0) {
    console.log(`${label}: OK (${left.size} variants)`);
    return;
  }

  console.error(`${label}: mismatch`);
  if (missingInRight.length > 0) {
    console.error(`  Missing in TypeScript: ${missingInRight.join(', ')}`);
  }
  if (missingInLeft.length > 0) {
    console.error(`  Missing in Rust: ${missingInLeft.join(', ')}`);
  }
  process.exit(1);
}

const rustRequests = extractRustVariants(dtoRust, 'APIRequest');
const rustResponses = extractRustVariants(dtoRust, 'APIResponse');

const tsRequests = extractTsTypeNames(dtoTs, 'APIRequest', 'APIResponse');
const tsResponses = extractTsTypeNames(dtoTs, 'APIResponse', 'DocumentId');

assertEqualSet('APIRequest variants', rustRequests, tsRequests);
assertEqualSet('APIResponse variants', rustResponses, tsResponses);
