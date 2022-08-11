import { render } from 'preact';
import { useState } from 'preact/hooks';
import { formatDate, formatDateHuman } from '../scripts/date';
import { DataDescription, DataDescriptionField, DocumentData } from './dto';
import { useQuery } from './hooks';
import { RPC } from './rpc';

const renderRoot = document.querySelector('main');
if (!renderRoot) {
  throw new Error('render root not found');
}

type SearchInputProps = {
  value: string;
  onSearch: (query: string) => void;
};
function SearchInput({ value, onSearch }: SearchInputProps) {
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

type RelTimeProps = {
  datetime: string;
  className?: string;
};

function RelTime({ datetime, className }: RelTimeProps) {
  const date = new Date(datetime);

  return (
    <time dateTime={datetime} title={formatDate(date)} className={className}>
      {formatDateHuman(date)}
    </time>
  );
}

type QueryErrorProps = {
  error: unknown;
};
function QueryError({ error }: QueryErrorProps) {
  if (!error) {
    return null;
  }

  return (
    <pre>
      <code>{String(error)}</code>
    </pre>
  );
}

type CatalogProps = {
  hidden?: boolean;
  onDocumentSelected: (documentId: string) => void;
};
function Catalog({ hidden, onDocumentSelected }: CatalogProps) {
  const [query, setQuery] = useState('');

  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.ListDocuments({ query }, abortSignal),
    [query]
  );

  const items = result?.documents.map((item) => (
    <div
      className="mb-4 cursor-pointer bg-zinc-100 px-4 py-2"
      key={item.id}
      onClick={() => onDocumentSelected(item.id)}
    >
      <div className="font-bold text-lg">
        [{item.documentType || 'erased'}] {item.title}
      </div>

      <RelTime className="font-mono text-sm" datetime={item.updatedAt} />
    </div>
  ));

  return (
    <div className="p-8" hidden={hidden}>
      <SearchInput value={query} onSearch={setQuery} />

      <QueryError error={error} />

      {inProgress && <div className="mb-8">Loading...</div>}

      {items}

      {result?.hasMore && <h2>HAS MORE</h2>}
    </div>
  );
}

type DocumentFieldProps = {
  field: DataDescriptionField;
  value: unknown;
};
function DocumentField({ field, value }: DocumentFieldProps) {
  return (
    <div className="mb-8">
      {field.name}: {value}
    </div>
  );
}

type DocumentFieldsProps = {
  data: DocumentData;
  dataDescription: DataDescription;
  subtype: string;
};
function DocumentFields({ data, dataDescription, subtype }: DocumentFieldsProps) {
  const fields = dataDescription.fields.filter(
    (field) => field.for_subtypes?.includes(subtype) ?? true
  );

  return (
    <>
      {fields.map((field) => (
        <DocumentField key={field.name} field={field} value={data[field.name]} />
      ))}
    </>
  );
}

type DocumentViewerProps = {
  documentId: string;
};
function DocumentViewer({ documentId }: DocumentViewerProps) {
  const { result, error, inProgress } = useQuery(
    (abortSignal) => RPC.GetDocument({ id: documentId }, abortSignal),
    []
  );

  return (
    <div>
      <QueryError error={error} />

      {inProgress && <div className="mb-8">Loading...</div>}

      {result && (
        <DocumentFields
          data={result.data}
          dataDescription={result.dataDescription}
          subtype={result.subtype}
        />
      )}
    </div>
  );
}

function Workspace() {
  const [documentId, setDocumentId] = useState<string>();

  return (
    <div>
      <Catalog hidden={Boolean(documentId)} onDocumentSelected={setDocumentId} />

      {documentId && <DocumentViewer key={documentId} documentId={documentId} />}
    </div>
  );
}

render(<Workspace />, renderRoot);
