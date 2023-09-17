import {
  ATTACHMENT_DOCUMENT_TYPE,
  DEFAULT_SUBTYPE,
  DocumentSubtype,
  DocumentType,
  ERASED_DOCUMENT_TYPE,
} from 'dto';
import { EmptyObj } from './index';

export type DataSchema = {
  modules: DataDescription[];
};

export type DataDescription = {
  document_type: DocumentType;
  subtypes?: DocumentSubtype[];
  fields: DataDescriptionField[];
};

export type DataDescriptionField = {
  name: string;
  field_type: FieldType;
  mandatory: boolean;
  readonly: boolean;
  for_subtypes?: DocumentSubtype[];
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
    (module) => module.document_type === documentType
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

export function getFieldDescriptions(
  documentType: DocumentType,
  subtype?: DocumentSubtype
): DataDescriptionField[] {
  const dataDescription = getDataDescription(documentType);

  if (subtype === undefined) {
    return dataDescription.fields;
  }

  return dataDescription.fields.filter((field) => isFieldActive(field, subtype));
}

export function isFieldActive(field: DataDescriptionField, subtype: DocumentSubtype): boolean {
  return field.for_subtypes?.includes(subtype) ?? true;
}

export function getDefaultSubtype(documentType: DocumentType): DocumentSubtype {
  const dataDescription = getDataDescription(documentType);

  return dataDescription.subtypes?.[0] ?? DEFAULT_SUBTYPE;
}

export function isAttachment(documentType: DocumentType) {
  return documentType === ATTACHMENT_DOCUMENT_TYPE;
}

export function isErasedDocument(documentType: DocumentType) {
  return documentType === ERASED_DOCUMENT_TYPE;
}

export function isImageAttachment(subtype: DocumentSubtype) {
  return subtype === 'image';
}

export function isAudioAttachment(subtype: DocumentSubtype) {
  return subtype === 'audio';
}

export function formatDocumentType(documentType: DocumentType, subtype?: DocumentSubtype): string {
  if (isErasedDocument(documentType)) {
    return 'erased';
  }

  if (subtype) {
    return `${documentType}/${subtype}`;
  }

  return documentType;
}
