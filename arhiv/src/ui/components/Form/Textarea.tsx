import { useEffect, useRef } from 'react';
import { Callback } from 'utils';

function autoGrowTextarea(textarea: HTMLTextAreaElement): Callback {
  // preserve height between updates
  const updateHeight = () => {
    const parent = textarea.parentElement;

    if (!parent) {
      throw new Error("Textarea doesn't have a parent element");
    }

    parent.style.height = `${parent.scrollHeight}px`;

    textarea.style.height = 'auto';

    textarea.style.height = `${textarea.scrollHeight}px`;

    parent.style.height = 'auto';
  };

  updateHeight();

  window.addEventListener('resize', updateHeight, { passive: true });

  return () => {
    window.removeEventListener('resize', updateHeight);
  };
}

interface Props {
  name: string;
  required?: boolean;
  value: string;
  onChange: (value: string) => void;
  autoGrow?: boolean;
}
export function Textarea({ name, required, value, onChange, autoGrow }: Props) {
  const textareaRef = useRef<HTMLTextAreaElement | null>(null);

  useEffect(() => {
    if (autoGrow && textareaRef.current) {
      return autoGrowTextarea(textareaRef.current);
    }
  }, [autoGrow, value]);

  return (
    <textarea
      ref={textareaRef}
      className="w-full block"
      name={name}
      required={required}
      value={value}
      onChange={(e) => {
        onChange(e.currentTarget.value);
      }}
    ></textarea>
  );
}
