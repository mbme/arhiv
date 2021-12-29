import { URL } from 'node:url';
import { Context } from './context';

function isImageUrl(urlStr: string): boolean {
  const url = new URL(urlStr);

  if (url.pathname.endsWith('.png') || url.pathname.endsWith('.jpg')) {
    return true;
  }

  return false;
}

export async function extractImage(urlStr: string, context: Context): Promise<boolean> {
  if (!isImageUrl(urlStr)) {
    return false;
  }

  await context.channel.createAttachment(urlStr);

  return true;
}
