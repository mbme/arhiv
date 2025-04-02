import { JSONObj, NominalType, Obj } from 'utils';
import { DataSchema } from 'utils/schema';

export type APIRequest =
  | {
      typeName: 'ListDocuments';
      documentTypes: DocumentType[];
      query: string;
      page: number;
    }
  | {
      typeName: 'GetDocuments';
      ids: DocumentId[];
    }
  | {
      typeName: 'GetStatus';
    }
  | {
      typeName: 'GetDocument';
      id: DocumentId;
    }
  | {
      typeName: 'ParseMarkup';
      markup: string;
    }
  | {
      typeName: 'SaveDocument';
      lockKey: DocumentLockKey;
      id: DocumentId;
      data: DocumentData;
      collections: DocumentId[];
    }
  | {
      typeName: 'CreateDocument';
      documentType: DocumentType;
      data: DocumentData;
      collections: DocumentId[];
    }
  | {
      typeName: 'EraseDocument';
      id: DocumentId;
    }
  | {
      typeName: 'ListDir';
      dir?: string;
      showHidden: boolean;
    }
  | {
      typeName: 'CreateAsset';
      filePath: string;
      removeFile: boolean;
    }
  | {
      typeName: 'Commit';
    }
  | {
      typeName: 'LockDocument';
      id: DocumentId;
    }
  | {
      typeName: 'UnlockDocument';
      id: DocumentId;
      lockKey?: DocumentLockKey;
      forceUnlock?: boolean;
    }
  | {
      typeName: 'ReorderCollectionRefs';
      collectionId: DocumentId;
      id: DocumentId;
      newPos: number;
    }
  | {
      typeName: 'CreateArhiv';
      password: string;
    }
  | {
      typeName: 'LockArhiv';
    }
  | {
      typeName: 'UnlockArhiv';
      password: string;
      secret: true;
    }
  | {
      typeName: 'ImportKey';
      encryptedKey: string;
      password: string;
      secret: true;
    }
  | {
      typeName: 'ExportKey';
      password: string;
      exportPassword: string;
      secret: true;
    };

export type APIResponse =
  | {
      typeName: 'ListDocuments';
      documents: ListDocumentsResult[];
      hasMore: boolean;
    }
  | {
      typeName: 'GetDocuments';
      documents: GetDocumentsResult[];
    }
  | {
      typeName: 'GetStatus';
      status: string;
    }
  | {
      typeName: 'GetDocument';
      id: DocumentId;
      documentType: DocumentType;
      title: string;
      updatedAt: string;
      data: DocumentData;
      backrefs: DocumentBackref[];
      collections: DocumentBackref[];
      refs: DocumentId[];
      snapshotsCount: number;
    }
  | {
      typeName: 'ParseMarkup';
      ast: MarkupElement;
    }
  | {
      typeName: 'SaveDocument';
      errors?: SaveDocumentErrors;
    }
  | {
      typeName: 'CreateDocument';
      id?: DocumentId;
      errors?: SaveDocumentErrors;
    }
  | {
      typeName: 'EraseDocument';
    }
  | {
      typeName: 'ListDir';
      dir: string;
      entries: DirEntry[];
    }
  | {
      typeName: 'CreateAsset';
      id: DocumentId;
    }
  | {
      typeName: 'Commit';
      committedIds: DocumentId[];
    }
  | {
      typeName: 'LockDocument';
      lockKey: DocumentLockKey;
    }
  | {
      typeName: 'UnlockDocument';
    }
  | {
      typeName: 'ReorderCollectionRefs';
    }
  | {
      typeName: 'CreateArhiv';
    }
  | {
      typeName: 'LockArhiv';
    }
  | {
      typeName: 'UnlockArhiv';
    }
  | {
      typeName: 'ImportKey';
    }
  | {
      typeName: 'ExportKey';
      key: string;
      qrcodeSvgBase64: string;
      htmlPage: string;
    };

export type DocumentId = NominalType<string, 'DocumentId'>;
export type DocumentType = NominalType<string, 'DocumentType'>;
export type DocumentLockKey = NominalType<string, 'DocumentLockKey'>;

export type DocumentDTO = Omit<Extract<APIResponse, { typeName: 'GetDocument' }>, 'typeName'>;

export const ERASED_DOCUMENT_TYPE = '' as DocumentType;
export const ASSET_DOCUMENT_TYPE = 'asset' as DocumentType;
export const PROJECT_DOCUMENT_TYPE = 'project' as DocumentType;
export const NOTE_DOCUMENT_TYPE = 'note' as DocumentType;
export const TASK_DOCUMENT_TYPE = 'task' as DocumentType;
export const BOOK_DOCUMENT_TYPE = 'book' as DocumentType;
export const GAME_DOCUMENT_TYPE = 'game' as DocumentType;
export const CONTACT_DOCUMENT_TYPE = 'contact' as DocumentType;
export const FILM_DOCUMENT_TYPE = 'film' as DocumentType;
export const TRACK_DOCUMENT_TYPE = 'track' as DocumentType;
export const EMPTY_DATA: DocumentData = {};

export type GetDocumentsResult<D = DocumentData> = {
  id: DocumentId;
  documentType: DocumentType;
  title: string;
  updatedAt: string;
  data: D;
};

export type ListDocumentsResult<D = DocumentData> = {
  id: DocumentId;
  documentType: DocumentType;
  title: string;
  updatedAt: string;
  data: D;
  cover?: DocumentId;
};

export type DocumentData = JSONObj;

export type DocumentBackref = {
  id: DocumentId;
  documentType: DocumentType;
  title: string;
};

export type DocumentFieldErrors = Obj<string[]>;

export type SaveDocumentErrors = {
  documentErrors: string[];
  fieldErrors: DocumentFieldErrors;
};

export type DirEntry =
  | {
      typeName: 'Dir';
      name: string;
      path: string;
      isReadable: boolean;
    }
  | {
      typeName: 'File';
      name: string;
      path: string;
      isReadable: boolean;
      size: number;
    }
  | {
      typeName: 'Symlink';
      name: string;
      path: string;
      isReadable: boolean;
      linksTo: string;
      size?: number;
    };

export type FileEntry = Extract<DirEntry, { typeName: 'File' }>;

export type MarkupElement =
  | {
      typeName: 'Document';
      children: MarkupElement[];
    }
  | {
      typeName: 'Text';
      range: Range;
      value: string;
    }
  | {
      typeName: 'Code';
      range: Range;
      value: string;
    }
  | {
      typeName: 'Html';
      range: Range;
      value: string;
    }
  | {
      typeName: 'FootnoteReference';
      range: Range;
      label: string;
    }
  | {
      typeName: 'SoftBreak';
      range: Range;
    }
  | {
      typeName: 'HardBreak';
      range: Range;
    }
  | {
      typeName: 'Rule';
      range: Range;
    }
  | {
      typeName: 'TaskListMarker';
      range: Range;
      checked: boolean;
    }
  | {
      typeName: 'Paragraph';
      range: Range;
      children: MarkupElement[];
    }
  | {
      typeName: 'Heading';
      range: Range;
      level: 'H1' | 'H2' | 'H3' | 'H4' | 'H5' | 'H6';
      children: MarkupElement[];
    }
  | {
      typeName: 'BlockQuote';
      range: Range;
      children: MarkupElement[];
    }
  | {
      typeName: 'CodeBlock';
      range: Range;
      kind: 'Indented' | { Fenced: string };
      children: MarkupElement[];
    }
  | {
      typeName: 'List';
      range: Range;
      first_item_number?: number;
      children: MarkupElement[];
    }
  | {
      typeName: 'ListItem';
      range: Range;
      children: MarkupElement[];
    }
  | {
      typeName: 'FootnoteDefinition';
      range: Range;
      label: string;
      children: MarkupElement[];
    }
  | {
      typeName: 'Table';
      range: Range;
      alignments: Array<'None' | 'Left' | 'Center' | 'Right'>;
      children: MarkupElement[];
    }
  | {
      typeName: 'TableHead';
      range: Range;
      children: MarkupElement[];
    }
  | {
      typeName: 'TableRow';
      range: Range;
      children: MarkupElement[];
    }
  | {
      typeName: 'TableCell';
      range: Range;
      children: MarkupElement[];
    }
  | {
      typeName: 'Emphasis';
      range: Range;
      children: MarkupElement[];
    }
  | {
      typeName: 'Strong';
      range: Range;
      children: MarkupElement[];
    }
  | {
      typeName: 'Strikethrough';
      range: Range;
      children: MarkupElement[];
    }
  | {
      typeName: 'Link';
      range: Range;
      link_type: LinkType;
      url: string;
      children: MarkupElement[];
    }
  | {
      typeName: 'Image';
      range: Range;
      link_type: LinkType;
      url: string;
      children: MarkupElement[];
    };

export type Range = {
  start: number;
  end: number;
};

type LinkType =
  | 'Inline'
  | 'Reference'
  | 'ReferenceUnknown'
  | 'Collapsed'
  | 'CollapsedUnknown'
  | 'Shortcut'
  | 'ShortcutUnknown'
  | 'Autolink'
  | 'Email';

export function throwBadMarkupElement(value: never): never;
export function throwBadMarkupElement(value: MarkupElement) {
  throw new Error(`Unknown MarkupElement: ${value.typeName}`);
}

export type ProjectData = {
  name: string;
  description: string;
  tasks: DocumentId[];
};

export type TaskStatus = 'InProgress' | 'Todo' | 'Done' | 'Cancelled';

export type TaskData = {
  title: string;
  status: TaskStatus;
};

export type ArhivUIConfig = {
  storageDir: string;
  basePath: string;
  schema: DataSchema;
  useLocalStorage: boolean;
  minPasswordLength: number;
  arhivMissing: boolean;
  arhivKeyMissing: boolean;
  arhivLocked: boolean;
};
