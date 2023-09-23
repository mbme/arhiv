import { JSONObj, NominalType, Obj } from 'utils';

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
      id: DocumentId;
      subtype: DocumentSubtype;
      data: DocumentData;
      collections: DocumentId[];
    }
  | {
      typeName: 'CreateDocument';
      documentType: DocumentType;
      subtype: DocumentSubtype;
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
      typeName: 'CreateAttachment';
      filePath: string;
      moveFile: boolean;
    }
  | {
      typeName: 'UploadFile';
      base64Data: string;
      fileName: string;
    }
  | {
      typeName: 'Scrape';
      url: string;
    }
  | {
      typeName: 'CommitOrSync';
    };

export type APIResponse =
  | {
      typeName: 'ListDocuments';
      documents: ListDocumentsResult[];
      hasMore: boolean;
    }
  | {
      typeName: 'GetDocuments';
      documents: ListDocumentsResult[];
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
      subtype: DocumentSubtype;
      updatedAt: string;
      data: DocumentData;
      backrefs: DocumentBackref[];
      collections: DocumentBackref[];
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
      typeName: 'CreateAttachment';
      id: DocumentId;
    }
  | {
      typeName: 'UploadFile';
      id: DocumentId;
    }
  | {
      typeName: 'Scrape';
      documents: ListDocumentsResult[];
    }
  | {
      typeName: 'CommitOrSync';
    };

export type DocumentId = NominalType<string, 'DocumentId'>;
export type DocumentType = NominalType<string, 'DocumentType'>;
export type DocumentSubtype = NominalType<string, 'DocumentSubtype'>;

export type DocumentDTO = Omit<Extract<APIResponse, { typeName: 'GetDocument' }>, 'typeName'>;

export const ERASED_DOCUMENT_TYPE = '' as DocumentType;
export const ATTACHMENT_DOCUMENT_TYPE = 'attachment' as DocumentType;
export const DEFAULT_SUBTYPE = '' as DocumentSubtype;
export const EMPTY_DATA: DocumentData = {};

export type ListDocumentsResult = {
  id: DocumentId;
  documentType: DocumentType;
  subtype: DocumentSubtype;
  title: string;
  updatedAt: string;
};

export type DocumentData = JSONObj;

export type DocumentBackref = {
  id: DocumentId;
  documentType: DocumentType;
  subtype: DocumentSubtype;
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
