export type Obj<T = string> = Record<string, T | undefined>;
export type EmptyObj = Obj<never>;
export type JSONValue =
  | null
  | string
  | number
  | boolean
  | { [prop: string]: JSONValue }
  | JSONValue[];
export type JSONObj = Obj<JSONValue>;

export type ArrayElement<ArrayType extends readonly unknown[]> =
  ArrayType extends readonly (infer ElementType)[] ? ElementType : never;

export type Callback = () => void;

// eslint-disable-next-line @typescript-eslint/no-empty-function
export const noop = (): void => {};

let _newIdState = 0;
export const newId = (): number => (_newIdState += 1);

export const ensure = (condition: unknown, message: string) => {
  if (!condition) {
    throw new Error(`assertion failed: condition is "${String(condition)}"; ${message}`);
  }
};

export function setQueryParam(urlStr: string, param: string, value: string | undefined): string {
  const url = new URL(urlStr, window.location.href);

  if (value === undefined) {
    url.searchParams.delete(param);
  } else {
    url.searchParams.set(param, value);
  }

  return url.toString();
}

export function updateQueryParam(param: string, value: string | undefined): void {
  const searchParams = new URLSearchParams(window.location.search);

  if (value) {
    searchParams.set(param, value);
  } else {
    searchParams.delete(param);
  }

  window.history.replaceState({}, '', '?' + searchParams.toString());
}

export function pickRandomElement<T>(arr: T[]): T {
  const pos = Math.floor(Math.random() * arr.length);

  return arr[pos];
}

export function cx(
  ...args: Array<string | null | undefined | false | Obj<string | null | undefined | boolean>>
): string {
  const result = [];

  for (const arg of args) {
    if (!arg) {
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

export function formatDocumentType(documentType: string, subtype?: string): string {
  if (subtype) {
    return `${documentType}/${subtype}`;
  }

  return documentType;
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
  value?: string | boolean | null
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

export const debounce = <Args extends any[], F extends (...args: Args) => void>(
  func: F,
  waitFor: number
): F => {
  let timeoutId: number;

  const debounced = (...args: Args) => {
    window.clearTimeout(timeoutId);
    timeoutId = window.setTimeout(() => func(...args), waitFor);
  };

  return debounced as F;
};

export function formDataToObject(fd: FormData): Record<string, string> {
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
