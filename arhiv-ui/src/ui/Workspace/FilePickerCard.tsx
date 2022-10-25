import { useCardContext } from './workspace-reducer';
import { CardContainer } from './CardContainer';
import { FilePicker } from '../components/FilePicker/FilePicker';

export function FilePickerCard() {
  const context = useCardContext();

  return (
    <>
      <CardContainer.Topbar
        left={<span className="section-heading text-lg">Add file</span>}
        right={<CardContainer.CloseButton />}
      />

      <FilePicker
        onAttachmentCreated={(documentId) => {
          context.replace({
            variant: 'document',
            documentId,
          });
        }}
      />
    </>
  );
}
