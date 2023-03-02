import { useEffect, useState } from 'preact/hooks';
import { cx } from 'utils';
import { HTMLVFormFieldElement } from 'components/Form/v-form-field';
import { CodemirrorEditor } from './CodemirrorEditor';

type Props = {
  className?: string;
  name: string;
  defaultValue?: string;
  placeholder?: string;
  disabled: boolean;
  readonly: boolean;
  required: boolean;
};
export function Editor({
  className,
  name,
  defaultValue,
  placeholder,
  disabled,
  readonly,
  required,
}: Props) {
  const [editor, setEditor] = useState<CodemirrorEditor>();
  const [fieldEl, setFieldEl] = useState<HTMLVFormFieldElement | null>(null);

  useEffect(() => {
    if (!fieldEl) {
      return;
    }

    const editor = new CodemirrorEditor(fieldEl, fieldEl.value?.toString() ?? '', {
      onBlur: () => {
        fieldEl.value = editor.getValue();
      },
      onChange: () => {
        if (!editor.isFocused()) {
          fieldEl.value = editor.getValue();
        }
      },
    });

    if (fieldEl.hasAttribute('autofocus')) {
      editor.focus();
    }

    setEditor(editor);

    return () => {
      setEditor(undefined);

      editor.destroy();
    };
  }, [fieldEl]);

  useEffect(() => {
    if (!editor) {
      return;
    }

    editor.setDisabled(disabled);
    editor.setReadonly(readonly);
    editor.setPlaceholder(placeholder ?? '');
  }, [editor, disabled, readonly, placeholder]);

  return (
    <v-form-field
      className={cx('editor-container', className)}
      ref={(el) => setFieldEl(el as HTMLVFormFieldElement | null)}
      name={name}
      defaultValue={JSON.stringify(defaultValue ?? '')}
      disabled={disabled}
      readonly={readonly}
      required={required}
      onChange={(e) => editor?.setValue(e.value as string)}
    />
  );
}
