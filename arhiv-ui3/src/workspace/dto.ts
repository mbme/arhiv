import { Obj } from '../scripts/utils';

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
    };

export type ListDocumentsResult = {
  id: string;
  documentType: string;
  subtype: string;
  title: string;
  updatedAt: string;
};

export type DocumentData = Obj<unknown>;
