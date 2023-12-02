export type Obj<T = string> = Record<string, T | undefined>;
export type EmptyObj = Obj<never>;

export type JSONValue =
  | null
  | string
  | number
  | boolean
  | { [prop: string]: JSONValue }
  | JSONValue[];
export type JSONObj = { [prop: string]: JSONValue };

export type ArrayElement<ArrayType extends readonly unknown[]> =
  ArrayType extends readonly (infer ElementType)[] ? ElementType : never;

export type Callback = () => void;

// https://dnlytras.com/blog/nominal-types
declare const __nominal__type: unique symbol;
export type NominalType<Type, Identifier> = Type & {
  readonly [__nominal__type]: Identifier;
};

// eslint-disable-next-line @typescript-eslint/no-empty-function
export const noop = (): void => {};

const _idPrefix = Math.random();
let _newIdState = 0;
export const newId = (): string => {
  _newIdState += 1;

  return `id/${_idPrefix}/${_newIdState}`;
};

export const ensure = (condition: unknown, message: string) => {
  if (!condition) {
    throw new Error(`assertion failed: condition is "${String(condition)}"; ${message}`);
  }
};

export function getQueryParam(name: string) {
  const params = new URLSearchParams(window.location.search);

  return params.get(name);
}

export function setQueryParam(param: string, value: string | undefined, replace = false): void {
  const url = new URL(window.location.href);

  if (value === undefined) {
    url.searchParams.delete(param);
  } else {
    url.searchParams.set(param, value);
  }

  if (replace) {
    window.history.replaceState({}, '', url.toString());
  } else {
    window.history.pushState({}, '', url.toString());
  }
}

export function cx(
  ...args: Array<string | null | undefined | boolean | Obj<string | null | undefined | boolean>>
): string {
  const result = [];

  for (const arg of args) {
    if (!arg || arg === true) {
      continue;
    }

    if (typeof arg === 'string') {
      result.push(arg);
      continue;
    }

    for (const [objKey, objVal] of Object.entries(arg)) {
      if (objVal) {
        result.push(objKey);
      }
    }
  }

  return result.join(' ');
}

export function px(value: number) {
  return `${value}px`;
}

export function getDocumentUrl(documentId: string): string {
  return `${window.location.origin}${window.BASE_PATH}?id=${documentId}`;
}

export function getSessionValue<T extends JSONValue>(key: string, defaultValue: T): T {
  const value = sessionStorage.getItem(key);

  if (value === null) {
    return defaultValue;
  }

  return JSON.parse(value) as T;
}

export function setSessionValue(key: string, value: JSONValue) {
  sessionStorage.setItem(key, JSON.stringify(value));
}

export function removeSessionValue(key: string) {
  sessionStorage.removeItem(key);
}

const BYTES_SIZES = ['B', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
export function formatBytes(bytes: number): string {
  if (!bytes) {
    return '0';
  }

  const power = Math.floor(Math.log(bytes) / Math.log(1024));

  const value = (bytes / Math.pow(1024, power)).toFixed(2);
  return `${value} ${BYTES_SIZES[power]}`;
}

export function setElementAttribute(
  el: HTMLElement,
  attribute: string,
  value?: string | boolean | null,
) {
  if (typeof value === 'string') {
    el.setAttribute(attribute, value);
    return;
  }

  if (value === true) {
    el.setAttribute(attribute, 'true');
    return;
  }

  el.removeAttribute(attribute);
}

// TODO get rid of any
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const debounce = <Args extends any[], F extends (...args: Args) => void>(
  func: F,
  waitForMs: number,
): F => {
  let timeoutId: number;

  const debounced = (...args: Args) => {
    window.clearTimeout(timeoutId);
    timeoutId = window.setTimeout(() => func(...args), waitForMs);
  };

  return debounced as F;
};

// throttle with trailing execution
// TODO get rid of any
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export const throttle = <Args extends any[], F extends (...args: Args) => void>(
  func: F,
  waitForMs: number,
): F => {
  let timeoutId: number | undefined;
  let pendingArgs: Args | undefined;

  const throttled = (...args: Args) => {
    if (timeoutId !== undefined) {
      pendingArgs = args; // save args for trailing execution
      return;
    }

    func(...args);

    timeoutId = window.setTimeout(() => {
      timeoutId = undefined;

      // trailing execution
      if (pendingArgs) {
        const args = pendingArgs;
        pendingArgs = undefined;
        throttled(...args);
      }
    }, waitForMs);
  };

  return throttled as F;
};

// FIXME improve types, handle multiple fields
export function formDataToObject(fd: FormData): Record<string, string | undefined> {
  const result: Record<string, string> = {};

  for (const [key, value] of fd.entries()) {
    if (value instanceof File) {
      throw new Error('unsupported: FormData contains a File');
    }

    if (result.hasOwnProperty(key)) {
      throw new Error(`unsupported: FormData contains duplicate key "${key}"`);
    }

    result[key] = value;
  }

  return result;
}

function _copyTextToClipbard(text: string): Promise<void> {
  if (document.hasFocus()) {
    return navigator.clipboard.writeText(text);
  }

  return new Promise((resolve, reject) => {
    const handler = () => {
      navigator.clipboard.writeText(text).then(resolve, reject);
    };

    window.addEventListener('focus', handler, { once: true });
  });
}

export function copyTextToClipbard(text: string): Promise<void> {
  return _copyTextToClipbard(text).then(
    () => {
      console.log('Copied text "%s" to clipboard"', text);
    },
    (e) => {
      console.error('Failed to copy text "%s" to clipboard', text, e);
    },
  );
}

/**
 * Check if needle fuzzy matches haystack.
 * @see https://github.com/bevacqua/fuzzysearch
 */
export function fuzzySearch(needle: string, haystack: string, ignoreCase = true): boolean {
  if (ignoreCase) {
    return fuzzySearch(needle.toLowerCase(), haystack.toLowerCase(), false);
  }

  const nlen = needle.length;

  // if needle is empty then it matches everything
  if (!nlen) {
    return true;
  }

  const hlen = haystack.length;
  if (nlen > hlen) {
    return false;
  }

  if (nlen === hlen) {
    return needle === haystack;
  }

  // eslint-disable-next-line no-labels
  outer: for (let i = 0, j = 0; i < nlen; i += 1) {
    const nch = needle.charCodeAt(i);
    while (j < hlen) {
      const char = haystack.charCodeAt(j);

      j += 1;

      if (char === nch) {
        continue outer; // eslint-disable-line no-labels
      }
    }

    return false;
  }

  return true;
}

export function fileAsBase64(file: File) {
  const reader = new FileReader();

  return new Promise<string>((resolve, reject) => {
    reader.onload = () => {
      const data = reader.result as string;

      const dataInBase64 = data.slice(data.indexOf(',') + 1);

      resolve(dataInBase64);
    };
    reader.onerror = reject;
    reader.onabort = reject;

    reader.readAsDataURL(file);
  });
}
