import { ComponentChildren, Ref, VNode } from 'preact';

export type JSXChildren = ComponentChildren;
export type JSXRef<T> = Ref<T>;
export type JSXElement = VNode<any> | null;

export function setJSXRef<T>(ref: JSXRef<T>, value: T | null) {
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
