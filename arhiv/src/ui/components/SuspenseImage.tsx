import { useLayoutEffect, useRef } from 'react';
import { Callback, cx } from 'utils';
import { useSuspenseImage } from 'utils/suspense';

type Props = {
  src: string;
  alt: string;
  className: string;
  onClick?: Callback;
};
export function SuspenseImage({ src, alt, className, onClick }: Props) {
  const img = useSuspenseImage(src);

  img.alt = alt;
  img.className = className;

  const containerRef = useRef<HTMLElement | null>(null);

  useLayoutEffect(() => {
    const containerEl = containerRef.current;
    if (!containerEl) {
      return;
    }

    containerEl.appendChild(img);

    return () => {
      containerEl.removeChild(img);
    };
  }, [img]);

  return (
    <span
      ref={containerRef}
      className={cx('block', {
        'cursor-pointer': Boolean(onClick),
      })}
      onClick={onClick}
    />
  );
}
