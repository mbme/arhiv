import { HTMLVEditorElement } from '../../../scripts/v-editor';
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
  const controlRef = useFormField<HTMLVEditorElement>(name, (editor) => editor.value);

  return <v-editor ref={controlRef} name={name} value={initialValue} />;
}
