import { cx, JSONValue } from '../../utils';
import { DataDescriptionField, FieldType } from '../../schema';
import { Checkbox } from '../Form/Checkbox';
import { useGettersContext } from '../Form/Form';
import { Select } from '../Form/Select';

function parseRefsList(refs: string): string[] | null {
  if (refs.trim().length === 0) {
    return null;
  }

  return refs
    .replaceAll(',', ' ')
    .split(' ')
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

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
  const getters = useGettersContext();

  if ('MarkupString' in fieldType) {
    return (
      <v-editor
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
        innerRef={(el) => {
          if (!el) {
            return;
          }
          getters.set(el, () => el.checked);
        }}
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
      <input
        className="field"
        ref={(el) => {
          if (!el) {
            return;
          }

          getters.set(el, () => el.value.trim() || null);
        }}
        type="text"
        name={name}
        defaultValue={initialValue as string | undefined}
        readOnly={readonly}
        required={required}
        disabled={disabled}
      />
    );
  }

  if ('RefList' in fieldType) {
    return (
      <input
        className="field"
        ref={(el) => {
          if (!el) {
            return;
          }

          getters.set(el, () => parseRefsList(el.value));
        }}
        type="text"
        name={name}
        defaultValue={(initialValue as string[] | undefined)?.join(', ') ?? undefined}
        readOnly={readonly}
        required={required}
        disabled={disabled}
      />
    );
  }

  if ('NaturalNumber' in fieldType) {
    return (
      <input
        className="field"
        ref={(el) => {
          if (!el) {
            return;
          }

          getters.set(el, () => (el.value ? Number.parseInt(el.value, 10) : null));
        }}
        type="number"
        min={0}
        step={1}
        name={name}
        defaultValue={(initialValue as number | undefined)?.toString() ?? undefined}
        readOnly={readonly}
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
  ignoreReadonly: boolean;
  disabled: boolean;
  errors?: string[];
};
export function DocumentEditorField({
  field,
  initialValue,
  ignoreReadonly,
  disabled,
  errors = [],
}: DocumentEditorFieldProps) {
  return (
    <label
      className={cx('flex flex-wrap justify-between items-center gap-y-3 py-3', {
        'has-errors': errors.length > 0,
      })}
      hidden={disabled}
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
