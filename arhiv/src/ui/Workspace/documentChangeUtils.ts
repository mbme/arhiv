import { useEffect } from 'react';
import { DocumentId } from 'dto';
import { Callback } from 'utils';
import { useLatestRef } from 'utils/hooks';

class DocumentChangeEvent extends CustomEvent<Set<DocumentId>> {
  public static EVENT_NAME = 'documentChangeEvent';

  constructor(ids: Set<DocumentId>) {
    super(DocumentChangeEvent.EVENT_NAME, { detail: ids });
  }
}

export function dispatchDocumentChangeEvent(ids: DocumentId[]) {
  document.dispatchEvent(new DocumentChangeEvent(new Set(ids)));
}

export function useDocumentChangeHandler(handler: (ids: Set<DocumentId>) => void) {
  const handlerRef = useLatestRef(handler);

  useEffect(() => {
    const onDocumentChange = (e: Event) => {
      handlerRef.current((e as DocumentChangeEvent).detail);
    };

    document.addEventListener(DocumentChangeEvent.EVENT_NAME, onDocumentChange);

    return () => {
      document.removeEventListener(DocumentChangeEvent.EVENT_NAME, onDocumentChange);
    };
  }, [handlerRef]);
}

export function useDocumentChange(ids: DocumentId[], onChange: Callback) {
  useDocumentChangeHandler((updatedDocumentsIds) => {
    const someReferencedDocumentsUpdated = ids.some((id) => updatedDocumentsIds.has(id));

    if (someReferencedDocumentsUpdated) {
      onChange();
    }
  });
}
