import { useEffect, useState } from 'react';
import { cx } from 'utils';
import { useDebouncedCallback } from 'utils/hooks';
import { Icon } from 'components/Icon';

type SearchInputProps = {
  autofocus?: boolean;
  initialValue: string;
  onSearch: (query: string) => void;
  busy?: boolean;
  placeholder?: string;
  debounceMs?: number;
  className?: string;
};

export function SearchInput({
  className,
  autofocus,
  initialValue,
  onSearch,
  busy,
  placeholder = 'Type something',
  debounceMs = 0,
}: SearchInputProps) {
  const [inputEl, setInputEl] = useState<HTMLInputElement | null>(null);

  useEffect(() => {
    if (inputEl && autofocus) {
      inputEl.focus();
    }
  }, [inputEl, autofocus]);

  const onSearchDebounced = useDebouncedCallback(onSearch, debounceMs);

  return (
    <form
      className={cx('relative', className)}
      onSubmit={(e) => {
        e.preventDefault();
      }}
    >
      <Icon
        variant={busy ? 'spinner' : 'search'}
        className="absolute top-3 left-3 pointer-events-none"
      />

      <input
        type="search"
        name="pattern"
        className="w-full pl-10"
        autoFocus={autofocus || undefined}
        defaultValue={initialValue}
        onChange={(e) => {
          onSearchDebounced(e.currentTarget.value);
        }}
        placeholder={placeholder}
        autoComplete="off"
        ref={setInputEl}
      />
    </form>
  );
}
