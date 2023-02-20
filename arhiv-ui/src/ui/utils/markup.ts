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
