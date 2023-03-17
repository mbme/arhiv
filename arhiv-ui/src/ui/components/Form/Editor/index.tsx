import { startTransition, useEffect, useRef, useState } from 'react';
import { cx } from 'utils';
import { createLink, createRefUrl } from 'utils/markup';
import { useUpdateEffect } from 'utils/hooks';
import { HTMLVFormFieldElement, FormField } from 'components/Form/FormField';
import { canPreview } from 'components/Ref';
import { Markup } from 'components/Markup';
import { IconButton } from 'components/Button';
import { SuspenseBoundary } from 'components/SuspenseBoundary';
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
  const defaultPreview = defaultValue.length > 0;
  const [preview, setPreview] = useState(defaultPreview);

  const fieldRef = useRef<HTMLVFormFieldElement | null>(null);
  const editorRef = useRef<CodemirrorEditor | null>(null);

  useEffect(() => {
    const fieldEl = fieldRef.current;
    if (!fieldEl) {
      throw new Error('field is missing');
    }

    const form = fieldEl.form;
    if (!form) {
      throw new Error('form is missing');
    }

    const editor = new CodemirrorEditor(fieldEl, fieldEl.value?.toString() ?? '', {
      onChange: () => {
        fieldEl.inputValue(editor.getValue());
      },
    });

    editorRef.current = editor;

    const onReset = () => {
      editor.setValue(fieldEl.value as string);
    };

    form.addEventListener('reset', onReset);

    return () => {
      editorRef.current = null;

      form.removeEventListener('reset', onReset);
      editor.destroy();
    };
  }, []);

  useEffect(() => {
    const editor = editorRef.current;
    if (!editor) {
      throw new Error('Editor is missing');
    }

    editor.setDisabled(disabled);
    editor.setReadonly(readonly);
    editor.setPlaceholder(placeholder ?? '');
  }, [disabled, readonly, placeholder]);

  // focus editor if editing except when the preview is initially false
  useUpdateEffect(() => {
    const editor = editorRef.current;
    if (!editor) {
      throw new Error('Editor is missing');
    }

    if (!preview) {
      editor.focus();
    }
  }, [preview]);

  useEffect(() => {
    const fieldEl = fieldRef.current;
    if (!fieldEl) {
      throw new Error('field is missing');
    }

    const form = fieldEl.form;
    if (!form) {
      throw new Error('field form is missing');
    }

    const onFormSubmit = () => {
      startTransition(() => {
        setPreview(defaultPreview);
      });
    };

    form.addEventListener('submit', onFormSubmit);

    return () => {
      form.removeEventListener('submit', onFormSubmit);
    };
  }, [defaultPreview]);

  return (
    <div className={cx('editor-container group', className)}>
      <FormField
        id={id}
        hidden={preview}
        innerRef={fieldRef}
        name={name}
        defaultValue={defaultValue}
        disabled={disabled}
        required={required}
        tabIndex={-1}
        onFocus={() => {
          editorRef.current?.focus();
        }}
      />

      <SuspenseBoundary>
        {preview && <Markup markup={editorRef.current?.getValue() ?? defaultValue} />}
      </SuspenseBoundary>

      <div className="sticky bottom-8 float-right mr-4 mt-1 flex gap-3">
        {!preview && (
          <AddRefButton
            className="bg-indigo-100 drop-shadow-md"
            onDocumentSelected={(id, documentType, subtype) => {
              const editor = editorRef.current;
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
            onClick={() => {
              setPreview(false);
            }}
          />
        ) : (
          <IconButton
            icon="eye"
            className="bg-indigo-100 drop-shadow-md"
            onClick={() => {
              setPreview(true);
            }}
          />
        )}
      </div>
    </div>
  );
}
