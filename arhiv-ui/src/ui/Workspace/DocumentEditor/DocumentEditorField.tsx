import { useEffect, useRef } from 'preact/hooks';
import { cx, JSONValue } from 'utils';
import { DataDescriptionField, FieldType } from 'utils/schema';
import { DocumentId } from 'dto';
import { Checkbox } from 'components/Form/Checkbox';
import { Select } from 'components/Form/Select';
import { Editor } from 'components/Form/Editor';
import { RefInput } from 'components/Form/RefInput';
import { useCardContext } from 'Workspace/workspace-reducer';

type ValueEditorProps = {
  name: string;
  fieldType: FieldType;
  initialValue?: JSONValue;
  required: boolean;
  readonly: boolean;
  disabled: boolean;
};
function ValueEditor({
  name,
  fieldType,
  initialValue,
  required,
  readonly,
  disabled,
}: ValueEditorProps) {
  const { openDocument } = useCardContext();

  if ('MarkupString' in fieldType) {
    return (
      <Editor
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
        className="field"
        name={name}
        initialValue={initialValue as string | undefined}
        options={fieldType.Enum}
        readonly={readonly}
        required={required}
        disabled={disabled}
      />
    );
  }

  if ('Flag' in fieldType) {
    return (
      <Checkbox
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
        className="field"
        documentTypes={[fieldType.Ref]}
        name={name}
        defaultValue={initialValue as DocumentId | undefined}
        readonly={readonly}
        required={required}
        disabled={disabled}
        onRefClick={openDocument}
      />
    );
  }

  if ('RefList' in fieldType) {
    return (
      <RefInput
        className="field"
        documentTypes={[fieldType.RefList]}
        name={name}
        defaultValue={initialValue as DocumentId[] | undefined}
        readonly={readonly}
        required={required}
        disabled={disabled}
        multiple
        onRefClick={openDocument}
      />
    );
  }

  if ('NaturalNumber' in fieldType) {
    return (
      <input
        type="number"
        className="field"
        min={0}
        step={1}
        name={name}
        defaultValue={(initialValue as number | undefined)?.toString()}
        readonly={readonly}
        required={required}
        disabled={disabled}
      />
    );
  }

  return (
    <input
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

type DocumentEditorFieldProps = {
  field: DataDescriptionField;
  initialValue?: JSONValue;
  autofocus: boolean;
  ignoreReadonly: boolean;
  disabled: boolean;
  errors?: string[];
};
export function DocumentEditorField({
  field,
  initialValue,
  autofocus,
  ignoreReadonly,
  disabled,
  errors = [],
}: DocumentEditorFieldProps) {
  const labelRef = useRef<HTMLLabelElement>(null);

  useEffect(() => {
    if (autofocus) {
      labelRef.current?.focus();
    }
  }, [autofocus]);

  return (
    <label
      className={cx('flex flex-wrap justify-between items-center gap-y-3 py-3', {
        'has-errors': errors.length > 0,
      })}
      hidden={disabled}
      ref={labelRef}
    >
      <h5 className="form-field-heading mr-8 relative">
        {field.name}
        {field.mandatory && (
          <span className="text-blue-500 text-xl absolute top-[-5px] pl-1">*</span>
        )}
      </h5>

      <ValueEditor
        name={field.name}
        fieldType={field.field_type}
        initialValue={initialValue}
        readonly={field.readonly && !ignoreReadonly}
        required={field.mandatory}
        disabled={disabled}
      />

      {errors.map((error, index) => (
        <div key={index} className="text-red-500 text-xs pl-1 my-2 w-full">
          {error}
        </div>
      ))}
    </label>
  );
}
