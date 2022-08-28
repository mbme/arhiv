import { JSONObj, Obj } from '../scripts/utils';

export type WorkspaceRequest =
  | {
      typeName: 'ListDocuments';
      query: string;
      page: number;
    }
  | {
      typeName: 'GetStatus';
    }
  | {
      typeName: 'GetDocument';
      id: string;
    }
  | {
      typeName: 'RenderMarkup';
      markup: string;
    }
  | {
      typeName: 'SaveDocument';
      id: string;
      subtype: string;
      data: DocumentData;
    }
  | {
      typeName: 'CreateDocument';
      documentType: string;
      subtype: string;
      data: DocumentData;
    }
  | {
      typeName: 'EraseDocument';
      id: string;
    }
  | {
      typeName: 'ListDir';
      dir?: string;
      showHidden: boolean;
    }
  | {
      typeName: 'CreateAttachment';
      filePath: string;
    };

export type WorkspaceResponse =
  | {
      typeName: 'ListDocuments';
      documents: ListDocumentsResult[];
      hasMore: boolean;
    }
  | {
      typeName: 'GetStatus';
      status: string;
    }
  | {
      typeName: 'GetDocument';
      id: string;
      documentType: string;
      title: string;
      subtype: string;
      updatedAt: string;
      data: DocumentData;
      backrefs: DocumentBackref[];
    }
  | {
      typeName: 'RenderMarkup';
      html: string;
    }
  | {
      typeName: 'SaveDocument';
      errors?: SaveDocumentErrors;
    }
  | {
      typeName: 'CreateDocument';
      id?: string;
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
      id: string;
    };

export type ListDocumentsResult = {
  id: string;
  documentType: string;
  subtype: string;
  title: string;
  updatedAt: string;
};

export type DocumentData = JSONObj;

export type DocumentBackref = {
  id: string;
  documentType: string;
  subtype: string;
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
