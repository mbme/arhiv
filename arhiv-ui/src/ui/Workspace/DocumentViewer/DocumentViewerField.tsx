import { JSXChildren } from 'utils/jsx';
import { DataDescriptionField } from 'utils/schema';
import { Markup } from 'components/Markup';
import { RefContainer } from 'components/Ref';
import { useCardContext } from '../workspace-reducer';

type FieldValueProps = {
  field: DataDescriptionField;
  value: unknown;
};

export function FieldValue({ field, value }: FieldValueProps) {
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
      <div className="w-full">
        {(value as string[]).map((id) => (
          <div key={id}>
            <RefContainer id={id} onClick={() => open({ variant: 'document', documentId: id })} />
          </div>
        ))}
      </div>
    );
  }

  return <>{String(value)}</>;
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
