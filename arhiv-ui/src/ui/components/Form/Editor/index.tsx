import { useEffect, useRef, useState } from 'react';
import { cx } from 'utils';
import { createLink, createRefUrl } from 'utils/markup';
import { HTMLVFormFieldElement, FormField } from 'components/Form/FormField';
import { canPreview } from 'components/Ref';
import { Markup } from 'components/Markup';
import { IconButton } from 'components/Button';
import { CodemirrorEditor } from './CodemirrorEditor';
import { AddRefButton } from './AddRefButton';
import { SuspenseBoundary } from 'components/SuspenseBoundary';

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

  // skip autofocus in case preview is immediately false
  const skipFocus = useRef(true);
  useEffect(() => {
    if (!editor) {
      return;
    }

    if (!preview && !skipFocus.current) {
      editor.focus();
    }
    skipFocus.current = false;
  }, [editor, preview]);

  return (
    <div className={cx('editor-container group', className)}>
      <FormField
        id={id}
        hidden={preview}
        innerRef={setFieldEl}
        name={name}
        defaultValue={defaultValue}
        disabled={disabled}
        required={required}
        tabIndex={-1}
        onFocus={() => {
          editor?.focus();
        }}
        onChange={(value) => {
          editor?.setValue(value as string);
        }}
      />

      <SuspenseBoundary>
        {preview && <Markup markup={editor?.getValue() ?? defaultValue} />}
      </SuspenseBoundary>

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

        {preview ? (
          <IconButton
            icon="pencil-square"
            className="bg-indigo-100 drop-shadow-md invisible opacity-0 group-hover:visible group-hover:opacity-100 transition-opacity"
            onClick={() => setPreview(!preview)}
          />
        ) : (
          <IconButton
            icon="eye"
            className="bg-indigo-100 drop-shadow-md"
            onClick={() => setPreview(!preview)}
          />
        )}
      </div>
    </div>
  );
}
