import { Editor, initEditor } from '../../../scripts/v-editor/editor';
import { useFormField } from './Form';

type MarkupEditorProps = {
  name: string;
  initialValue?: string;
  readonly: boolean;
  mandatory: boolean;
};
export function MarkupEditor({ name, initialValue, readonly, mandatory }: MarkupEditorProps) {
  const controlRef = useFormField<Editor>(name, (editor) => editor.state.doc.toString());

  const initContainer = (containerEl: HTMLDivElement | null) => {
    if (!containerEl || controlRef.current) {
      return;
    }

    controlRef.current = initEditor(containerEl, initialValue || '', !readonly);
  };

  return <div ref={initContainer} />;
}
