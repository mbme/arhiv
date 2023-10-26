import { MutableRefObject, ReactElement, ReactNode, Ref, RefCallback } from 'react';

export type JSXChildren = ReactNode;
export type JSXRef<T> = RefCallback<T> | MutableRefObject<T | null>;
export type JSXElement = ReactElement | null;

export function setJSXRef<T>(ref: JSXRef<T> | null, value: T | null) {
  if (ref instanceof Function) {
    ref(value);
  } else if (ref) {
    ref.current = value;
  }
}

export function mergeRefs<T, R extends T>(...refs: ReadonlyArray<JSXRef<T> | undefined>): Ref<R> {
  return (value) => {
    for (const ref of refs) {
      if (ref) {
        setJSXRef(ref, value);
      }
    }
  };
}
