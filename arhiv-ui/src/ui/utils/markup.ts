import { DocumentId } from 'dto';

export function createRefUrl(id: DocumentId) {
  return `ref:${id}`;
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
