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

  return (
    <span
      className={cx('block', {
        'hover:scale-105 hover:translate-y-2 transition-transform': Boolean(onClick),
      })}
      onClick={onClick}
      ref={(el) => {
        if (el && !el.contains(img)) {
          el.appendChild(img);
        }
      }}
    />
  );
}
