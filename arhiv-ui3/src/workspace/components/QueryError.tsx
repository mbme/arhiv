type QueryErrorProps = {
  error: unknown;
};

export function QueryError({ error }: QueryErrorProps) {
  if (!error) {
    return null;
  }

  return (
    <pre>
      <code>{String(error)}</code>
    </pre>
  );
}
