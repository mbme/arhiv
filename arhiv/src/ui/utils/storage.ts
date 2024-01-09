import { JSONValue } from 'utils';

interface Storage {
  getValue<T extends JSONValue>(key: string, defaultValue: T): T;

  setValue(key: string, value: JSONValue): void;

  removeValue(key: string): void;

  clear(): void;
}

class SessionStorage implements Storage {
  getValue<T extends JSONValue>(key: string, defaultValue: T): T {
    const value = sessionStorage.getItem(key);

    if (value === null) {
      return defaultValue;
    }

    return JSON.parse(value) as T;
  }

  setValue(key: string, value: JSONValue): void {
    sessionStorage.setItem(key, JSON.stringify(value));
  }

  removeValue(key: string): void {
    sessionStorage.removeItem(key);
  }

  clear(): void {
    sessionStorage.clear();
  }
}

class LocalStorage implements Storage {
  getValue<T extends JSONValue>(key: string, defaultValue: T): T {
    const value = localStorage.getItem(key);

    if (value === null) {
      return defaultValue;
    }

    return JSON.parse(value) as T;
  }

  setValue(key: string, value: JSONValue): void {
    localStorage.setItem(key, JSON.stringify(value));
  }

  removeValue(key: string): void {
    localStorage.removeItem(key);
  }

  clear(): void {
    localStorage.clear();
  }
}

export const storage = window.FEATURES.use_local_storage
  ? new LocalStorage()
  : new SessionStorage();
