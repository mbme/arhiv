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

export type Suspender<T> = { read: () => T };
export function suspensify<T>(promise: Promise<T>): Suspender<T> {
  type SuspenseStatus = { type: 'resolved'; value: T } | { type: 'rejected'; value: any };

  let status: SuspenseStatus | undefined;

  const pendingPromise = promise.then(
    (result) => {
      status = { type: 'resolved', value: result };
    },
    (err) => {
      status = { type: 'rejected', value: err };
    }
  );

  return {
    read() {
      if (!status) {
        throw pendingPromise;
      }

      if (status.type === 'rejected') {
        throw status.value;
      }

      return status.value;
    },
  };
}
