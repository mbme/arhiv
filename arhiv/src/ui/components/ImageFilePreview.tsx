import { useLayoutEffect, useState } from 'react';
import { cx, isImage } from 'utils';

type Props = {
  file: File;
  className?: string;
};
export function ImageFilePreview({ file, className }: Props) {
  const [image, setImage] = useState<HTMLImageElement | null>(null);

  useLayoutEffect(() => {
    if (!image || !isImage(file.type)) {
      return;
    }

    const url = URL.createObjectURL(file);

    image.src = url;

    return () => {
      image.src = '';
      URL.revokeObjectURL(url);
    };
  }, [image, file]);

  if (!isImage(file.type)) {
    return null;
  }

  return (
    <img
      ref={setImage}
      className={cx(className, 'cursor-pointer')}
      onClick={(e) => {
        const url = e.currentTarget.src;
        window.open(url, '_blank');
      }}
    />
  );
}
