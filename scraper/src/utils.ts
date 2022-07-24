import * as chrono from 'chrono-node';

export const promiseTimeout = (timeoutMs: number) =>
  new Promise<void>((resolve) => setTimeout(resolve, timeoutMs));

export const getLocationURL = () => new URL(window.location.href);

export const getPathSegments = (url: URL) =>
  url.pathname.split('/').filter((segment) => segment.length > 0);

export const parseHumanDate = (dateStr: string): Date | undefined => {
  const date: Date | null = chrono.parseDate(dateStr);

  return date ?? undefined;
};

export const getEl = <T extends Element = HTMLElement>(
  root: HTMLElement | Document,
  selector: string,
  description: string
): T => {
  const el = root.querySelector(selector);

  if (!el) {
    throw new Error(`can't find element '${description}' using selector '${selector}'`);
  }

  return el as T;
};
