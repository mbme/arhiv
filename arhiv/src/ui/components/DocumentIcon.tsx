import {
  DocumentType,
  DocumentSubtype,
  TASK_DOCUMENT_TYPE,
  NOTE_DOCUMENT_TYPE,
  PROJECT_DOCUMENT_TYPE,
  ERASED_DOCUMENT_TYPE,
  BOOK_DOCUMENT_TYPE,
  GAME_DOCUMENT_TYPE,
  CONTACT_DOCUMENT_TYPE,
  FILM_DOCUMENT_TYPE,
  TRACK_DOCUMENT_TYPE,
} from 'dto';
import { Icon, IconVariant } from 'components/Icon';
import { isCollection } from 'utils/schema';

type Props = {
  documentType: DocumentType;
};
export function DocumentIcon({ documentType }: Props) {
  let variant: IconVariant = isCollection(documentType) ? 'file-cabinet' : 'file-outline';

  if (documentType === NOTE_DOCUMENT_TYPE) {
    variant = 'note-text-outline';
  }

  if (documentType === TASK_DOCUMENT_TYPE) {
    variant = 'checkbox-marked-outline';
  }

  if (documentType === PROJECT_DOCUMENT_TYPE) {
    variant = 'task-list';
  }

  if (documentType === BOOK_DOCUMENT_TYPE) {
    variant = 'book-open';
  }

  if (documentType === GAME_DOCUMENT_TYPE) {
    variant = 'game';
  }

  if (documentType === FILM_DOCUMENT_TYPE) {
    variant = 'film';
  }

  if (documentType === TRACK_DOCUMENT_TYPE) {
    variant = 'musical-note';
  }

  if (documentType === CONTACT_DOCUMENT_TYPE) {
    variant = 'contact';
  }

  if (documentType === ERASED_DOCUMENT_TYPE) {
    variant = 'erase-document';
  }

  return <Icon variant={variant} className="w-auto h-auto text-zinc-500" />;
}
