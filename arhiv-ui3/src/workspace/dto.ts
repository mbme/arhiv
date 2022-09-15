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
      typeName: 'ParseMarkup';
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
    }
  | {
      typeName: 'Scrape';
      url: string;
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
      typeName: 'ParseMarkup';
      ast: MarkupElement;
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
    }
  | {
      typeName: 'Scrape';
      documents: ListDocumentsResult[];
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
      title: string;
      children: MarkupElement[];
    }
  | {
      typeName: 'Image';
      range: Range;
      link_type: LinkType;
      url: string;
      title: string;
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
