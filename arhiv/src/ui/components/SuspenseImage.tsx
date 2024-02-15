import { useSuspenseImage } from 'utils/suspense';

type Props = {
  src: string;
  alt: string;
  className: string;
};
export function SuspenseImage({ src, alt, className }: Props) {
  const img = useSuspenseImage(src);

  img.alt = alt;
  img.className = className;

  return (
    <span
      className="block"
      ref={(el) => {
        if (el && !el.contains(img)) {
          el.appendChild(img);
        }
      }}
    />
  );
}
