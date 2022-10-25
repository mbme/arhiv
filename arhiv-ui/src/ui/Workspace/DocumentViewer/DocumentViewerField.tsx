import { DataDescriptionField } from '../../utils/schema';
import { Markup } from '../../components/Markup';
import { RefContainer } from '../../components/Ref';
import { useCardContext } from '../workspace-reducer';

type DocumentViewerFieldProps = {
  field: DataDescriptionField;
  value: unknown;
};

function FieldValue({ field, value }: DocumentViewerFieldProps) {
  const { open } = useCardContext();

  if ('MarkupString' in field.field_type) {
    return (
      <Markup
        markup={value as string}
        onRefClick={(documentId) => open({ variant: 'document', documentId })}
      />
    );
  }

  if ('Ref' in field.field_type) {
    const id = value as string;

    return (
      <RefContainer
        key={id}
        id={id}
        attachmentPreview
        onClick={() => open({ variant: 'document', documentId: id })}
      />
    );
  }

  if ('RefList' in field.field_type) {
    return (
      <>
        {(value as string[]).map((id) => (
          <RefContainer
            key={id}
            id={id}
            onClick={() => open({ variant: 'document', documentId: id })}
          />
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
