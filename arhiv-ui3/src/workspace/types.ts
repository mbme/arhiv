export type JSXChildren = React.ReactNode | readonly React.ReactNode[];
export type JSXRef<T> = React.ForwardedRef<T>;
export type EffectCallback = React.EffectCallback;

export function setJSXRef<T>(ref: JSXRef<T>, value: T | null) {
  if (ref instanceof Function) {
    ref(value);
  } else if (ref) {
    ref.current = value;
  }
}
