import { useEffect, useId, useRef } from 'react';
import { cx, JSONValue } from 'utils';
import { DataDescriptionField, FieldType } from 'utils/schema';
import { Checkbox } from 'components/Form/Checkbox';
import { Select } from 'components/Form/Select';
import { Editor } from 'components/Form/Editor';
import { Ref, RefInput } from 'components/Form/RefInput';

type ValueEditorProps = {
  id: string;
  name: string;
  fieldType: FieldType;
  initialValue?: JSONValue;
  required: boolean;
  readonly: boolean;
  disabled?: boolean;
};
function ValueEditor({
  id,
  name,
  fieldType,
  initialValue,
  required,
  readonly,
  disabled = false,
}: ValueEditorProps) {
  if ('MarkupString' in fieldType) {
    return (
      <Editor
        id={id}
        className="field"
        name={name}
        defaultValue={initialValue as string | undefined}
        readonly={readonly}
        required={required}
        disabled={disabled}
      />
    );
  }

  if ('Enum' in fieldType) {
    return (
      <Select
        id={id}
        className="field"
        name={name}
        initialValue={(initialValue as string | undefined) ?? ''}
        options={['', ...fieldType.Enum]}
        readonly={readonly}
        required={required}
        disabled={disabled}
      />
    );
  }

  if ('Flag' in fieldType) {
    return (
      <Checkbox
        id={id}
        className="field"
        name={name}
        initialValue={initialValue === 'true'}
        readonly={readonly}
        required={required}
        disabled={disabled}
      />
    );
  }

  if ('Ref' in fieldType) {
    return (
      <RefInput
        id={id}
        className="field"
        documentTypes={fieldType.Ref}
        name={name}
        defaultValue={initialValue as Ref | undefined}
        readonly={readonly}
        required={required}
        disabled={disabled}
      />
    );
  }

  if ('RefList' in fieldType) {
    return (
      <RefInput
        id={id}
        className="field"
        documentTypes={fieldType.RefList}
        name={name}
        defaultValue={initialValue as Ref[] | undefined}
        readonly={readonly}
        required={required}
        disabled={disabled}
        multiple
      />
    );
  }

  if ('NaturalNumber' in fieldType) {
    return (
      <input
        id={id}
        type="number"
        className="field"
        min={0}
        step={1}
        name={name}
        defaultValue={(initialValue as number | undefined)?.toString()}
        readOnly={readonly}
        required={required}
        disabled={disabled}
      />
    );
  }

  return (
    <input
      id={id}
      className="field"
      type="text"
      name={name}
      defaultValue={initialValue as string}
      readOnly={readonly}
      required={required}
      disabled={disabled}
    />
  );
}

type DocumentFieldProps = {
  field: DataDescriptionField;
  initialValue?: JSONValue;
  autofocus: boolean;
  ignoreReadonly: boolean;
  errors?: string[];
};
export function DocumentField({
  field,
  initialValue,
  autofocus,
  ignoreReadonly,
  errors = [],
}: DocumentFieldProps) {
  const id = useId();
  const labelRef = useRef<HTMLLabelElement>(null);

  useEffect(() => {
    if (autofocus) {
      labelRef.current?.focus();
    }
  }, [autofocus]);

  return (
    <div
      className={cx('flex flex-wrap justify-between items-center gap-y-3 py-3', {
        'has-errors': errors.length > 0,
      })}
    >
      <label ref={labelRef} htmlFor={id}>
        <h5 className="form-field-heading mr-8 relative">
          {field.name}
          {field.mandatory && (
            <span className="text-blue-500 text-xl absolute top-[-5px] pl-1">*</span>
          )}
        </h5>
      </label>

      <ValueEditor
        id={id}
        name={field.name}
        fieldType={field.field_type}
        initialValue={initialValue}
        readonly={field.readonly && !ignoreReadonly}
        required={field.mandatory}
      />

      {errors.map((error, index) => (
        <div key={index} className="text-red-500 text-xs pl-1 my-2 w-full">
          {error}
        </div>
      ))}
    </div>
  );
}
