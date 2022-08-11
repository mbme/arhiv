type SearchInputProps = {
  value: string;
  onSearch: (query: string) => void;
};

export function SearchInput({ value, onSearch }: SearchInputProps) {
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        onSearch((e.target as HTMLFormElement).querySelector('input')!.value);
      }}
    >
      <input
        type="search"
        name="pattern"
        class="field w-full mb-8"
        value={value}
        placeholder="Type something"
        autofocus
      />
    </form>
  );
}
