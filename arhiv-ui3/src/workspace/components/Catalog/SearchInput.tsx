type SearchInputProps = {
  value: string;
  onSearch: (query: string) => void;
};

export function SearchInput({ value, onSearch }: SearchInputProps) {
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();

        const input = (e.target as HTMLFormElement).querySelector('input');
        if (!input) {
          throw new Error('unreachable: input must exist');
        }

        onSearch(input.value);
        input.blur();
      }}
    >
      <input
        type="search"
        name="pattern"
        class="field w-full mb-8"
        value={value}
        placeholder="Type something"
      />
    </form>
  );
}
