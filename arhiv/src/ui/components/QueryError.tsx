type QueryErrorProps = {
  error: unknown;
  children?: React.ReactNode;
};

export function QueryError({ error, children }: QueryErrorProps) {
  return (
    <pre className="bg-red-300 text-neutral-800 p-4 whitespace-pre-wrap break-words">
      <code>{String(error)}</code>
      {children}
    </pre>
  );
}
