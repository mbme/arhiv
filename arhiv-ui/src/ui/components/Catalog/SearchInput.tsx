import { useDebouncedCallback } from '../../utils/hooks';
import { Icon } from '../Icon';

type SearchInputProps = {
  initialValue: string;
  onSearch: (query: string) => void;
  busy: boolean;
};

export function SearchInput({ initialValue, onSearch, busy }: SearchInputProps) {
  const onSearchDebounced = useDebouncedCallback(onSearch, 400);

  return (
    <form
      className="form relative pt-1"
      onSubmit={(e) => {
        e.preventDefault();
      }}
    >
      <Icon
        variant={busy ? 'spinner' : 'search'}
        className="absolute top-4 left-3 pointer-events-none"
      />

      <input
        type="search"
        name="pattern"
        className="field w-full mb-4 pl-10"
        defaultValue={initialValue}
        onChange={(e) => {
          onSearchDebounced(e.currentTarget.value);
        }}
        placeholder="Type something"
        autoComplete="off"
      />
    </form>
  );
}
