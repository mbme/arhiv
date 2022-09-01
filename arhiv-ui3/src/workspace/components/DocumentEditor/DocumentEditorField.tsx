import { cx, JSONValue } from '../../../scripts/utils';
import { DataDescriptionField } from '../../schema';
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
  field: DataDescriptionField;
  initialValue?: JSONValue;
  ignoreReadonly: boolean;
  disabled: boolean;
};
function ValueEditor({ field, initialValue, ignoreReadonly, disabled }: ValueEditorProps) {
  const getters = useGettersContext();

  if ('MarkupString' in field.field_type) {
    return (
      <v-editor
        className="field"
        name={field.name}
        defaultValue={initialValue as string | undefined}
        readonly={field.readonly && !ignoreReadonly}
        required={field.mandatory}
        disabled={disabled}
      />
    );
  }

  if ('Enum' in field.field_type) {
    return (
      <Select
        className="field"
        name={field.name}
        initialValue={initialValue as string | undefined}
        options={field.field_type.Enum}
        readonly={field.readonly && !ignoreReadonly}
        required={field.mandatory}
        disabled={disabled}
      />
    );
  }

  if ('Flag' in field.field_type) {
    return (
      <Checkbox
        className="field"
        innerRef={(el) => {
          if (!el) {
            return;
          }
          getters.set(el, () => el.checked);
        }}
        name={field.name}
        initialValue={initialValue === 'true'}
        readonly={field.readonly && !ignoreReadonly}
        required={field.mandatory}
        disabled={disabled}
      />
    );
  }

  if ('Ref' in field.field_type) {
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
        name={field.name}
        defaultValue={initialValue as string | undefined}
        readonly={field.readonly && !ignoreReadonly}
        required={field.mandatory}
        disabled={disabled}
      />
    );
  }

  if ('RefList' in field.field_type) {
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
        name={field.name}
        defaultValue={(initialValue as string[] | undefined)?.join(', ') ?? undefined}
        readonly={field.readonly && !ignoreReadonly}
        required={field.mandatory}
        disabled={disabled}
      />
    );
  }

  if ('NaturalNumber' in field.field_type) {
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
        name={field.name}
        defaultValue={(initialValue as number | undefined)?.toString() ?? undefined}
        readonly={field.readonly && !ignoreReadonly}
        required={field.mandatory}
        disabled={disabled}
      />
    );
  }

  return (
    <input
      className="field"
      type="text"
      name={field.name}
      defaultValue={initialValue as string}
      readonly={field.readonly && !ignoreReadonly}
      required={field.mandatory}
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
    <label className={cx('block mb-12', { 'has-errors': errors.length > 0 })} hidden={disabled}>
      <h5 className="section-heading mb-2 relative">
        {field.name}
        {field.readonly && !ignoreReadonly && '(readonly)'}
        {field.mandatory && <span class="text-blue-500 text-xl absolute top-[-5px] pl-1">*</span>}
      </h5>

      <ValueEditor
        field={field}
        initialValue={initialValue}
        ignoreReadonly={ignoreReadonly}
        disabled={disabled}
      />

      {errors.map((error, index) => (
        <div key={index} class="text-red-500 text-xs pl-1 my-2">
          {error}
        </div>
      ))}
    </label>
  );
}
