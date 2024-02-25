import {
  ATTACHMENT_DOCUMENT_TYPE,
  DocumentType,
  ERASED_DOCUMENT_TYPE,
  PROJECT_DOCUMENT_TYPE,
} from 'dto';
import { EmptyObj } from './index';

export type DataSchema = {
  modules: DataDescription[];
};

export type DataDescription = {
  document_type: DocumentType;
  fields: DataDescriptionField[];
};

export type DataDescriptionField = {
  name: string;
  field_type: FieldType;
  mandatory: boolean;
  readonly: boolean;
};

export type FieldType =
  | { String: EmptyObj }
  | { MarkupString: EmptyObj }
  | { Flag: EmptyObj }
  | { NaturalNumber: EmptyObj }
  | { Ref: DocumentType }
  | { RefList: DocumentType }
  | { BLOBId: EmptyObj }
  | { Enum: string[] }
  | { Date: EmptyObj }
  | { Duration: EmptyObj }
  | { People: EmptyObj }
  | { Countries: EmptyObj };

export function getDocumentTypes(collections: boolean): DocumentType[] {
  return window.SCHEMA.modules
    .filter((module) => isModuleCollection(module) === collections)
    .map((module) => module.document_type)
    .sort();
}

export function getDataDescription(documentType: DocumentType): DataDescription {
  const dataDescription = window.SCHEMA.modules.find(
    (module) => module.document_type === documentType,
  );
  if (!dataDescription) {
    throw new Error(`Can't find data description for "${documentType}"`);
  }

  return dataDescription;
}

function isModuleCollection(module: DataDescription): boolean {
  return module.fields.some((field) => 'RefList' in field.field_type);
}

function isModuleCollectionForDocument(module: DataDescription, documentType: DocumentType) {
  return module.fields.some((field) => {
    if ('RefList' in field.field_type) {
      return field.field_type.RefList === documentType;
    }

    return false;
  });
}

export function getCollectionTypesForDocument(documentType: DocumentType) {
  return window.SCHEMA.modules
    .filter((module) => isModuleCollectionForDocument(module, documentType))
    .map((module) => module.document_type);
}

export function isCollection(documentType: DocumentType): boolean {
  return getDocumentTypes(true).includes(documentType);
}

export function isAttachment(documentType: DocumentType) {
  return documentType === ATTACHMENT_DOCUMENT_TYPE;
}

export function isProject(documentType: DocumentType) {
  return documentType === PROJECT_DOCUMENT_TYPE;
}

export function isErasedDocument(documentType: DocumentType) {
  return documentType === ERASED_DOCUMENT_TYPE;
}

export function formatDocumentType(documentType: DocumentType): string {
  if (isErasedDocument(documentType)) {
    return 'erased';
  }

  return documentType;
}
