import { DocumentId } from 'dto';

export const REF_LINK_PREFIX = 'ref:';

export function createRefUrl(id: DocumentId) {
  return `${REF_LINK_PREFIX}${id}`;
}

export function isRefUrl(value: string) {
  return value.startsWith(REF_LINK_PREFIX) && value.length > REF_LINK_PREFIX.length;
}

export function tryParseRefUrl(value: string): DocumentId | undefined {
  if (isRefUrl(value)) {
    return value.substring(REF_LINK_PREFIX.length) as DocumentId;
  }
}

export function createLink(url: string, description: string, preview = false) {
  if (!description && !preview) {
    return `<${url}>`;
  }

  if (preview) {
    return `![${description}](${url})`;
  }

  return `[${description}](${url})`;
}

export function getLineAt(value: string, pos: number): string {
  if (pos < 0 || pos >= value.length) {
    throw new Error(`wrong pos: ${pos}`);
  }

  let lineStart = value.lastIndexOf('\n', pos);
  if (lineStart === -1) {
    lineStart = 0;
  } else {
    lineStart += 1;
  }

  let lineEnd = value.indexOf('\n', pos + 1);
  if (lineEnd === -1) {
    lineEnd = value.length;
  }

  return value.slice(lineStart, lineEnd);
}
