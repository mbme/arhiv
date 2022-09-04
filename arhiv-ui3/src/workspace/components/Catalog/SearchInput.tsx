import { useEffect, useRef, useState } from 'react';
import { useDebouncedValue } from '../../hooks';
import { Icon } from '../Icon';

type SearchInputProps = {
  value: string;
  onSearch: (query: string) => void;
  busy: boolean;
};

export function SearchInput({ value: initialValue, onSearch, busy }: SearchInputProps) {
  const onSearchRef = useRef(onSearch);
  onSearchRef.current = onSearch;

  const [value, setValue] = useState(initialValue);
  const debouncedValue = useDebouncedValue(value, 400);

  useEffect(() => {
    onSearchRef.current(debouncedValue);
  }, [debouncedValue]);

  return (
    <form
      className="form relative"
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
        className="field w-full mb-4 pl-10"
        value={value}
        onChange={(e) => {
          setValue(e.currentTarget.value);
        }}
        placeholder="Type something"
        autoComplete="off"
      />
    </form>
  );
}
