import { ComponentChildren } from 'preact';

type CardTopbarProps = {
  children: ComponentChildren;
};
export function CardTopbar({ children }: CardTopbarProps) {
  return (
    <div className="flex gap-2 justify-between bg-neutral-200 py-2 mb-12 sticky top-0 z-10">
      {children}
    </div>
  );
}
