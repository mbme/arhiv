import { JSONValue } from '../../../scripts/utils';
import { DataDescriptionField } from '../../schema';
import {
  Flag,
  MarkupEditor,
  NaturalNumberInput,
  RefInput,
  RefListInput,
  Select,
  TextInput,
} from '../Form';

type ValueEditorProps = {
  field: DataDescriptionField;
  initialValue?: JSONValue;
};
function ValueEditor({ field, initialValue }: ValueEditorProps) {
  if ('MarkupString' in field.field_type) {
    return (
      <MarkupEditor
        name={field.name}
        initialValue={initialValue as string | undefined}
        readonly={field.readonly}
        mandatory={field.mandatory}
      />
    );
  }

  if ('Enum' in field.field_type) {
    return (
      <Select
        name={field.name}
        initialValue={initialValue as string | undefined}
        options={field.field_type.Enum}
        readonly={field.readonly}
        mandatory={field.mandatory}
      />
    );
  }

  if ('Flag' in field.field_type) {
    return (
      <Flag
        name={field.name}
        initialValue={initialValue === 'true'}
        readonly={field.readonly}
        mandatory={field.mandatory}
      />
    );
  }

  if ('Ref' in field.field_type) {
    return (
      <RefInput
        name={field.name}
        initialValue={initialValue as string | undefined}
        readonly={field.readonly}
        mandatory={field.mandatory}
      />
    );
  }

  if ('RefList' in field.field_type) {
    return (
      <RefListInput
        name={field.name}
        initialValue={initialValue as string[] | undefined}
        readonly={field.readonly}
        mandatory={field.mandatory}
      />
    );
  }

  if ('NaturalNumber' in field.field_type) {
    return (
      <NaturalNumberInput
        name={field.name}
        initialValue={initialValue as number | undefined}
        readonly={field.readonly}
        mandatory={field.mandatory}
      />
    );
  }

  return (
    <TextInput
      name={field.name}
      initialValue={initialValue as string}
      readonly={field.readonly}
      mandatory={field.mandatory}
    />
  );
}

type DocumentEditorFieldProps = {
  field: DataDescriptionField;
  initialValue?: JSONValue;
  hidden: boolean;
  errors?: string[];
};
export function DocumentEditorField({
  field,
  initialValue,
  hidden,
  errors = [],
}: DocumentEditorFieldProps) {
  return (
    <label className={`block mb-12 ${errors.length > 0 ? 'has-errors' : ''}`} hidden={hidden}>
      <h5 className="section-heading mb-2 relative">
        {field.name}
        {field.readonly && '(readonly)'}
        {field.mandatory && <span class="text-blue-500 text-xl absolute top-[-5px] pl-1">*</span>}
      </h5>

      <ValueEditor field={field} initialValue={initialValue} />

      {errors.map((error, index) => (
        <div key={index} class="text-red-500 text-xs pl-1 my-2">
          {error}
        </div>
      ))}
    </label>
  );
}
