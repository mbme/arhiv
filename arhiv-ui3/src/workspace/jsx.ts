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
