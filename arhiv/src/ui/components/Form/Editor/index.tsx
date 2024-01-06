import { useCallback, useEffect, useRef, useState } from 'react';
import { cx } from 'utils';
import { createLink, createRefUrl } from 'utils/markup';
import { useUpdateEffect } from 'utils/hooks';
import { HTMLVFormFieldElement, FormField } from 'components/Form/FormField';
import { FORM_VIEWPORT_CLASSNAME } from 'components/Form/Form';
import { canPreview } from 'components/Ref';
import { Markup, MarkupRef } from 'components/Markup';
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
  const defaultPreview = defaultValue.length > 0;
  const [preview, _setPreview] = useState(defaultPreview);

  const fieldRef = useRef<HTMLVFormFieldElement | null>(null);
  const markupRef = useRef<MarkupRef | null>(null);
  const editorRef = useRef<CodemirrorEditor | null>(null);
  const posRef = useRef<number | undefined>(undefined);

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

    editorRef.current = editor;

    return () => {
      editorRef.current = null;

      editor.destroy();
    };
  }, []);

  useEffect(() => {
    const editor = editorRef.current;
    if (!editor) {
      return;
    }

    editor.setDisabled(disabled);
    editor.setReadonly(readonly);
    editor.setPlaceholder(placeholder ?? '');
  }, [disabled, readonly, placeholder]);

  // focus editor if editing except when the preview is initially false
  useUpdateEffect(() => {
    if (!preview) {
      editorRef.current?.focus();
    }
  }, [preview]);

  const setPreview = useCallback((newPreview: boolean) => {
    _setPreview((oldPreview) => {
      if (oldPreview === newPreview) {
        return oldPreview;
      }

      const fieldEl = fieldRef.current;
      if (!fieldEl) {
        throw new Error('field is missing');
      }

      const viewportEl = fieldEl.closest<HTMLElement>(`.${FORM_VIEWPORT_CLASSNAME}`);
      if (!viewportEl) {
        throw new Error('form viewport element is missing');
      }

      if (newPreview) {
        posRef.current = editorRef.current?.getFirstVisiblePos(viewportEl);
        console.debug('first visible pos from editor', posRef.current);
      } else {
        const markupEl = markupRef.current;
        if (!markupEl) {
          throw new Error('markup element is missing');
        }

        posRef.current = markupEl.getFirstVisiblePos(viewportEl);
        console.debug('first visible pos from preview', posRef.current);
      }

      return newPreview;
    });
  }, []);

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
      setPreview((editorRef.current?.getValue().length ?? 0) > 0);
    };

    const onFormReset = () => {
      setPreview(defaultPreview);
    };

    form.addEventListener('submit', onFormSubmit);
    form.addEventListener('reset', onFormReset);

    return () => {
      form.removeEventListener('submit', onFormSubmit);
      form.removeEventListener('reset', onFormReset);
    };
  }, [defaultPreview, setPreview]);

  useEffect(() => {
    const pos = posRef.current;
    if (!pos) {
      return;
    }

    if (preview) {
      markupRef.current?.scrollToPos(pos);
    } else {
      editorRef.current?.scrollToPos(pos);
    }
  }, [preview]);

  return (
    <div className={cx('editor-container', className)}>
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
        onReset={() => {
          editorRef.current?.setValue(defaultValue);
        }}
      />

      {preview && <Markup ref={markupRef} markup={editorRef.current?.getValue() ?? defaultValue} />}

      <div className="sticky bottom-8 float-right mr-4 mt-1 flex gap-3">
        {!preview && (
          <AddRefButton
            className="bg-indigo-100 drop-shadow-md"
            onDocumentSelected={({ id, documentType, subtype }) => {
              editorRef.current?.replaceSelections((value) =>
                createLink(createRefUrl(id), value, canPreview(documentType, subtype)),
              );
            }}
          />
        )}

        {preview ? (
          <IconButton
            icon="pencil-square"
            className="edit-btn"
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
