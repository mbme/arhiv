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
  className?: string;
  fieldType: FieldType;
  initialValue?: JSONValue;
  required: boolean;
  readonly: boolean;
  disabled?: boolean;
};
function ValueEditor({
  id,
  name,
  className,
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
        name={name}
        className={className}
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
        name={name}
        className={className}
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
        name={name}
        className={className}
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
        documentTypes={fieldType.Ref}
        name={name}
        className={className}
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
        documentTypes={fieldType.RefList}
        name={name}
        className={className}
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
        min={0}
        step={1}
        name={name}
        className={className}
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
      type="text"
      name={name}
      className={className}
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

  const hasErrors = errors.length > 0;

  return (
    <div className="flex flex-wrap justify-between items-center gap-y-3 py-3">
      <label ref={labelRef} htmlFor={id}>
        <h5
          className={cx('form-field-heading mr-8 relative', {
            'has-errors': hasErrors,
          })}
        >
          {field.name}
          {field.mandatory && (
            <span className="text-blue-500 text-xl absolute top-[-5px] pl-1">*</span>
          )}
        </h5>
      </label>

      <ValueEditor
        id={id}
        name={field.name}
        className={cx({
          'var-form-error-border': hasErrors,
        })}
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
