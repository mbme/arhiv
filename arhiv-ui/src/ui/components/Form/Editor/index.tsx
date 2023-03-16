import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { cx, px } from 'utils';
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
  const [preview, setPreview] = useState(defaultValue.length > 0);

  const fieldRef = useRef<HTMLVFormFieldElement | null>(null);
  const editorRef = useRef<CodemirrorEditor | null>(null);

  useEffect(() => {
    const fieldEl = fieldRef.current;
    if (!fieldEl) {
      throw new Error('field is missing');
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

    editorRef.current = editor;

    return () => {
      editorRef.current = null;

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

  // skip autofocus in case preview is immediately false
  // useUpdateEffect
  useUpdateEffect(() => {
    const editor = editorRef.current;
    if (!editor) {
      throw new Error('Editor is missing');
    }

    if (!preview) {
      editor.focus();
    }
  }, [preview]);

  const containerRef = useRef<HTMLDivElement | null>(null);
  useLayoutEffect(() => {
    const containerEl = containerRef.current;
    if (!containerEl) {
      throw new Error('container el is missing');
    }

    containerEl.style.minHeight = '';

    return () => {
      containerEl.style.minHeight = px(containerEl.scrollHeight);
    };
  }, [preview]);

  return (
    <div ref={containerRef} className={cx('editor-container group', className)}>
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
        onChange={(value) => {
          editorRef.current?.setValue(value as string);
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
