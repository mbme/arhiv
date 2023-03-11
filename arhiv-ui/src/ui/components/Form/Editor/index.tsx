import { Suspense } from 'preact/compat';
import { useEffect, useState } from 'preact/hooks';
import { cx } from 'utils';
import { createLink, createRefUrl } from 'utils/markup';
import { HTMLVFormFieldElement } from 'components/Form/v-form-field';
import { canPreview } from 'components/Ref';
import { Icon } from 'components/Icon';
import { Markup } from 'components/Markup';
import { IconButton } from 'components/Button';
import { CodemirrorEditor } from './CodemirrorEditor';
import { AddRefButton } from './AddRefButton';

type Props = {
  id?: string;
  className?: string;
  name: string;
  defaultValue?: string;
  placeholder?: string;
  disabled: boolean;
  readonly: boolean;
  required: boolean;
};
export function Editor({
  id,
  className,
  name,
  defaultValue = '',
  placeholder,
  disabled,
  readonly,
  required,
}: Props) {
  const [preview, setPreview] = useState(defaultValue.length > 0);
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

  useEffect(() => {
    if (editor && !preview) {
      editor.focus();
    }
  }, [editor, preview]);

  return (
    <div className={cx('editor-container', className)}>
      <v-form-field
        id={id}
        hidden={preview}
        ref={(el) => setFieldEl(el as HTMLVFormFieldElement | null)}
        name={name}
        defaultValue={JSON.stringify(defaultValue)}
        disabled={disabled}
        readonly={readonly}
        required={required}
        tabIndex={-1}
        onFocus={() => {
          editor?.focus();
        }}
        onChange={(e) => {
          const value = e.value as string;
          editor?.setValue(value);
        }}
      />
      {preview && (
        <Suspense fallback={<Icon variant="spinner" className="mb-8" />}>
          <Markup markup={editor?.getValue() ?? defaultValue} />
        </Suspense>
      )}

      <div className="sticky bottom-8 float-right mr-4 mt-1 flex gap-3">
        {!preview && (
          <AddRefButton
            className="bg-indigo-100 drop-shadow-md"
            onDocumentSelected={(id, documentType, subtype) => {
              if (!editor) {
                throw new Error('editor is missing');
              }

              editor.replaceSelections((value) =>
                createLink(createRefUrl(id), value, canPreview(documentType, subtype))
              );

              editor.focus();
            }}
          />
        )}

        <IconButton
          icon={preview ? 'pencil-square' : 'eye'}
          className="bg-indigo-100 drop-shadow-md"
          onClick={() => setPreview(!preview)}
        />
      </div>
    </div>
  );
}
