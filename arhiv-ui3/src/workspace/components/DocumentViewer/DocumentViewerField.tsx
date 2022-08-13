import { DataDescriptionField } from '../../schema';
import { Markup } from '../Markup';
import { Ref } from '../Ref';

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

    return <Ref key={id} id={id} />;
  }

  if ('RefList' in field.field_type) {
    return (
      <>
        {(value as string[]).map((id) => (
          <Ref key={id} id={id} />
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
    <section className="mb-16">
      <h5 class="section-heading mb-4">{field.name}</h5>
      <FieldValue field={field} value={value} />
    </section>
  );
}
