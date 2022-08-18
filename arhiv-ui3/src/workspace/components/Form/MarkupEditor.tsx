import { useEffect, useRef } from 'preact/hooks';
import { Callback } from '../../../scripts/utils';
import { Editor, initEditor } from '../../../scripts/v-editor/editor';
import { useFormField } from './Form';

type MarkupEditorProps = {
  name: string;
  initialValue?: string;
  readonly: boolean;
  mandatory: boolean;
};
export function MarkupEditor({
  name,
  initialValue,
  readonly: _readonly, // TODO handle
  mandatory: _mandatory, // TODO handle
}: MarkupEditorProps) {
  const controlRef = useFormField<Editor>(name, (editor) => editor.state.doc.toString());
  const unsubRef = useRef<Callback>();

  const initContainer = (containerEl: HTMLDivElement | null) => {
    if (!containerEl || controlRef.current) {
      return;
    }

    const editor = initEditor(containerEl, initialValue || '');

    controlRef.current = editor;

    const label = containerEl.closest('label');
    const clickHandler = () => editor.focus();
    label?.addEventListener('click', clickHandler);
    unsubRef.current = () => label?.removeEventListener('click', clickHandler);
  };

  useEffect(() => {
    return () => unsubRef.current?.();
  }, []);

  return <div ref={initContainer} />;
}
