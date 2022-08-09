export type WorkspaceRequest =
  | {
      typeName: 'ListDocuments';
      query?: string;
    }
  | {
      typeName: 'GetStatus';
    };

export type WorkspaceResponse =
  | {
      typeName: 'ListDocuments';
      documents: string[];
    }
  | {
      typeName: 'GetStatus';
      status: string;
    };
