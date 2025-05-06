import { startTransition, useCallback, useEffect, useRef, useState } from 'react';
import { cx } from 'utils';
import { createLink, createRefUrl } from 'utils/markup';
import { useSignal, useUpdateEffect } from 'utils/hooks';
import { useAppController } from 'controller';
import { HTMLVFormFieldElement, FormField } from 'components/Form/FormField';
import { FORM_VIEWPORT_CLASSNAME } from 'components/Form/Form';
import { canPreview } from 'components/AssetPreview';
import { Markup, MarkupRef } from 'components/Markup';
import { IconButton } from 'components/Button';
import { DocumentPicker } from 'components/DocumentPicker';
import { CodemirrorEditor } from './CodemirrorEditor';

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
  const app = useAppController();

  const defaultPreview = defaultValue.length > 0;
  const [preview, _setPreview] = useState(defaultPreview);
  const [showPicker, setShowPicker] = useState(false);

  const fieldRef = useRef<HTMLVFormFieldElement<string> | null>(null);
  const markupRef = useRef<MarkupRef | null>(null);
  const editorRef = useRef<CodemirrorEditor | null>(null);
  const posRef = useRef<number | undefined>(undefined);
  const theme = useSignal(app.$theme);

  useEffect(() => {
    const fieldEl = fieldRef.current;
    if (!fieldEl) {
      throw new Error('field is missing');
    }

    const editor = new CodemirrorEditor(fieldEl, fieldEl.value?.toString() ?? '', {
      onChange: () => {
        fieldEl.inputValue(editor.getValue());
      },
      bottomPanel: (
        <div className="flex gap-4 justify-end py-1 pr-1">
          <IconButton
            icon="link"
            className="editor-btn"
            onClick={() => {
              setShowPicker(true);
            }}
          />

          <IconButton
            icon="eye"
            className="editor-btn"
            onClick={() => {
              setPreview(true);
            }}
          />
        </div>
      ),
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
    editor.setDarkMode(theme === 'dark');
  }, [disabled, readonly, placeholder, theme]);

  // focus editor if editing except when the preview is initially false
  useUpdateEffect(() => {
    if (!preview) {
      editorRef.current?.focus();
    }
  }, [preview]);

  const setPreview = useCallback((newPreview: boolean) => {
    // transition is needed to avoid a spinner when submitting the document
    startTransition(() => {
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

      {preview && (
        <IconButton
          icon="pencil-square"
          className="editor-edit-btn"
          onClick={() => {
            setPreview(false);
          }}
        />
      )}

      {showPicker && (
        <DocumentPicker
          hideOnSelect
          onSelected={({ id, documentType, data }) => {
            editorRef.current?.replaceSelections((value) =>
              createLink(createRefUrl(id), value, canPreview(documentType, data)),
            );
            setShowPicker(false);
          }}
          onCancel={() => {
            setShowPicker(false);
          }}
        />
      )}
    </div>
  );
}
