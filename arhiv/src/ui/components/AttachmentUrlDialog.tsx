import { Dialog } from 'components/Dialog';
import { Button } from 'components/Button';
import { Form } from 'components/Form/Form';

type Props = {
  onConfirm: (url: string) => void;
  onCancel: () => void;
};
export function AttachmentUrlDialog({ onConfirm, onCancel }: Props) {
  return (
    <Dialog onHide={onCancel} title="Create attachment from URL">
      <Form
        onSubmit={(data) => {
          onConfirm(data['url'] as string);
        }}
      >
        <div className="flex gap-2 mb-8">
          <input
            type="url"
            name="url"
            placeholder="Enter URL"
            className="grow"
            autoComplete="off"
            autoFocus
            data-autofocus
          />

          <Button type="submit" variant="primary">
            Confirm
          </Button>
        </div>
      </Form>
    </Dialog>
  );
}
