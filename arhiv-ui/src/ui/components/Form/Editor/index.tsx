import { startTransition, useEffect, useRef, useState } from 'react';
import { cx } from 'utils';
import { createLink, createRefUrl } from 'utils/markup';
import { useImmediateEffect, useUpdateEffect } from 'utils/hooks';
import { HTMLVFormFieldElement, FormField } from 'components/Form/FormField';
import { canPreview } from 'components/Ref';
import { Markup, MarkupRef } from 'components/Markup';
import { IconButton } from 'components/Button';
import { CodemirrorEditor } from './CodemirrorEditor';
import { AddRefButton } from './AddRefButton';
import { FORM_VIEWPORT_CLASSNAME } from '../Form';

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
  const markupRef = useRef<MarkupRef | null>(null);
  const [editor, setEditor] = useState<CodemirrorEditor>();

  useEffect(() => {
    const fieldEl = fieldRef.current;
    if (!fieldEl) {
      throw new Error('field is missing');
    }

    const editor = new CodemirrorEditor(fieldEl, fieldEl.value?.toString() ?? '', {
      onChange: () => {
        fieldEl.inputValue(editor.getValue());
      },
    });

    setEditor(editor);

    return () => {
      editor.destroy();
    };
  }, []);

  useEffect(() => {
    if (!editor) {
      return;
    }

    editor.setDisabled(disabled);
    editor.setReadonly(readonly);
    editor.setPlaceholder(placeholder ?? '');
  }, [editor, disabled, readonly, placeholder]);

  // focus editor if editing except when the preview is initially false
  useUpdateEffect(() => {
    if (!editor) {
      return;
    }

    if (!preview) {
      editor.focus();
    }
  }, [editor, preview]);

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

  const posRef = useRef<number | undefined>(undefined);
  useImmediateEffect(() => {
    const fieldEl = fieldRef.current;
    if (!fieldEl) {
      throw new Error('field is missing');
    }

    const viewportEl = fieldEl.closest<HTMLElement>(`.${FORM_VIEWPORT_CLASSNAME}`);
    if (!viewportEl) {
      throw new Error('form viewport element is missing');
    }

    if (preview) {
      posRef.current = editor?.getFirstVisiblePos(viewportEl);
      console.debug('first visible pos from editor', posRef.current);
    } else {
      const markupEl = markupRef.current;
      if (!markupEl) {
        throw new Error('markup element is missing');
      }

      posRef.current = markupEl.getFirstVisiblePos(viewportEl);
      console.debug('first visible pos from preview', posRef.current);
    }
  }, [preview]);

  useEffect(() => {
    const pos = posRef.current;
    if (!pos) {
      return;
    }

    if (preview) {
      markupRef.current?.scrollToPos(pos);
    } else {
      editor?.scrollToPos(pos);
    }
  }, [preview, editor]);

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
          editor?.focus();
        }}
        onReset={() => {
          editor?.setValue(defaultValue);
        }}
      />

      {preview && <Markup ref={markupRef} markup={editor?.getValue() ?? defaultValue} />}

      {editor && (
        <div className="sticky bottom-8 float-right mr-4 mt-1 flex gap-3">
          {!preview && (
            <AddRefButton
              className="bg-indigo-100 drop-shadow-md"
              onDocumentSelected={({ id, documentType, subtype }) => {
                editor.replaceSelections((value) =>
                  createLink(createRefUrl(id), value, canPreview(documentType, subtype))
                );
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
      )}
    </div>
  );
}
