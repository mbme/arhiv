import { DocumentId } from 'dto';
import { JSXChildren } from 'utils/jsx';
import { DataDescriptionField } from 'utils/schema';
import { Markup } from 'components/Markup';
import { RefContainer, RefListContainer } from 'components/Ref';

type FieldValueProps = {
  field: DataDescriptionField;
  value: unknown;
};

export function FieldValue({ field, value }: FieldValueProps) {
  if ('MarkupString' in field.field_type) {
    return <Markup markup={value as string} />;
  }

  if ('Ref' in field.field_type) {
    const id = value as DocumentId;

    return <RefContainer key={id} id={id} attachmentPreview />;
  }

  if ('RefList' in field.field_type) {
    const ids = value as DocumentId[];

    return (
      <div className="w-full flex flex-col">
        <div className="font-mono">{ids.length} items</div>
        <RefListContainer ids={ids} />
      </div>
    );
  }

  return <div className="break-words w-full">{String(value)}</div>;
}

type DocumentViewerFieldProps = {
  name: string;
  children: JSXChildren;
};
export function DocumentViewerField({ name, children }: DocumentViewerFieldProps) {
  return (
    <section className="py-3 flex flex-wrap justify-between align-center">
      <h5 className="form-field-heading mb-1">{name}</h5>
      {children}
    </section>
  );
}
