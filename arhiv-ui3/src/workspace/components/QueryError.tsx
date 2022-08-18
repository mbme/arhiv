type QueryErrorProps = {
  error: unknown;
};

export function QueryError({ error }: QueryErrorProps) {
  return (
    <pre className="border-red-500">
      <code>{String(error)}</code>
    </pre>
  );
}
