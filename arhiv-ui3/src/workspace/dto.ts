import { JSONObj, Obj } from '../scripts/utils';

export type WorkspaceRequest =
  | {
      typeName: 'ListDocuments';
      query: string;
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
      typeName: 'GetRef';
      id: string;
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
      subtype: string;
      updatedAt: string;
      data: DocumentData;
    }
  | {
      typeName: 'RenderMarkup';
      html: string;
    }
  | {
      typeName: 'GetRef';
      id: string;
      documentType: string;
      subtype: string;
      title: string;
    }
  | {
      typeName: 'SaveDocument';
      errors?: SaveDocumentErrors;
    }
  | {
      typeName: 'CreateDocument';
      id?: string;
      errors?: SaveDocumentErrors;
    };

export type ListDocumentsResult = {
  id: string;
  documentType: string;
  subtype: string;
  title: string;
  updatedAt: string;
};

export type DocumentData = JSONObj;

export type DocumentFieldErrors = Obj<string[]>;

export type SaveDocumentErrors = {
  documentErrors: string[];
  fieldErrors: DocumentFieldErrors;
};
