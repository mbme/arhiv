export type WorkspaceRequest =
  | {
      typeName: 'ListDocuments';
      query: string;
    }
  | {
      typeName: 'GetStatus';
    };

export type ListDocumentsResult = {
  id: string;
  documentType: string;
  title: string;
  updatedAt: string;
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
    };
