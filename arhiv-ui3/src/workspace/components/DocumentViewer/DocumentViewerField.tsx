import { DataDescriptionField } from '../../schema';
import { Markup } from '../Markup';
import { RefContainer } from '../Ref';

type DocumentViewerFieldProps = {
  field: DataDescriptionField;
  value: unknown;
};

function FieldValue({ field, value }: DocumentViewerFieldProps) {
  if ('MarkupString' in field.field_type) {
    return <Markup markup={value as string} />;
  }

  if ('Ref' in field.field_type) {
    const id = value as string;

    return <RefContainer key={id} id={id} attachmentPreview />;
  }

  if ('RefList' in field.field_type) {
    return (
      <>
        {(value as string[]).map((id) => (
          <RefContainer key={id} id={id} />
        ))}
      </>
    );
  }

  return <>{String(value)}</>;
}

export function DocumentViewerField({ field, value }: DocumentViewerFieldProps) {
  if (!value) {
    return null;
  }

  return (
    <section className="py-3 flex flex-wrap justify-between align-center">
      <h5 className="form-field-heading mb-1">{field.name}</h5>
      <FieldValue field={field} value={value} />
    </section>
  );
}
