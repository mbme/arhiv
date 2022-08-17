import { ComponentChildren } from 'preact';

type CardProps = {
  children: ComponentChildren;
};
export function Card({ children }: CardProps) {
  return <div className="w-[38rem] shrink-0 bg-white px-4 py-2 overflow-auto">{children}</div>;
}
