import { ComponentChildren } from 'preact';

type CardProps = {
  children: ComponentChildren;
};
export function Card({ children }: CardProps) {
  return <div className="max-w-lg bg-white px-4 py-2 max-h-screen overflow-auto">{children}</div>;
}
