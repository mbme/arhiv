type QueryErrorProps = {
  error: unknown;
};

export function QueryError({ error }: QueryErrorProps) {
  return (
    <pre className="bg-red-300 p-4">
      <code>{String(error)}</code>
    </pre>
  );
}
