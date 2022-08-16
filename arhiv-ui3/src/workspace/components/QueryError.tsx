type QueryErrorProps = {
  error: unknown;
};

export function QueryError({ error }: QueryErrorProps) {
  if (!error) {
    return null;
  }

  return (
    <pre className="border-red-500">
      <code>{String(error)}</code>
    </pre>
  );
}
