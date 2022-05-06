import { Obj } from './utils';

export type Id = string;

export type Revision = number;

export type Document = {
  id: Id;
  rev: Revision;
  prev_ref: Revision;
  document_type: string;
  subtype: string;
  created_at: string;
  updated_at: string;
  data: {
    [prop: string]: unknown;
  };
};

export type Filter = {
  page_offset?: number;
  page_size?: number;
  conditions?: Conditions;
  order?: OrderBy[];
};

export type ListPage = {
  items: Document[];
  has_more: boolean;
};

export type OrderBy =
  | {
      type: 'Field';
      selector: string;
      asc: boolean;
    }
  | {
      type: 'EnumField';
      selector: string;
      asc: boolean;
      enum_order: string[];
    }
  | {
      type: 'UpdatedAt';
      asc: boolean;
    };

export type Conditions = {
  field?: [string, string];
  search?: string;
  document_type?: string;
  document_ref?: Id;
  collection_ref?: Id;
  only_staged?: boolean;
};

export const RPC = new Proxy(
  {},
  {
    get(_, prop) {
      return async (params: Obj = {}) => {
        try {
          const response = await fetch('/api', {
            method: 'POST',
            body: JSON.stringify({
              type: prop as string,
              ...params,
            }),
          });

          const message = await response.text();

          if (!response.ok) {
            throw new Error(`API call failed: ${response.status}\n${message}`);
          }

          return JSON.parse(message) as Obj;
        } catch (e) {
          console.error(e);
          throw e;
        }
      };
    },
  }
) as {
  'ListDocuments': (params: { filter: Filter }) => Promise<{ page: ListPage }>;
  'Status': () => Promise<{ status: string }>;
};
