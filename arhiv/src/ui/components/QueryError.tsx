type QueryErrorProps = {
  error: unknown;
};

export function QueryError({ error }: QueryErrorProps) {
  return (
    <pre className="bg-red-300 text-neutral-800 p-4 whitespace-pre-wrap break-words">
      <code>{String(error)}</code>
    </pre>
  );
}
