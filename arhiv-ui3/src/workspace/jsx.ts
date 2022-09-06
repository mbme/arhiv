import { ComponentChildren, Ref } from 'preact';

export type JSXChildren = ComponentChildren;
export type JSXRef<T> = Ref<T>;

export function setJSXRef<T>(ref: JSXRef<T>, value: T | null) {
  if (ref instanceof Function) {
    ref(value);
  } else if (ref) {
    ref.current = value;
  }
}
