import { EmptyObj } from './index';

export type DataSchema = {
  modules: DataDescription[];
};

export type DataDescription = {
  document_type: string;
  subtypes?: string[];
  collection_of: 'None' | { Type: { document_type: string; field: string } };
  fields: DataDescriptionField[];
};

export type DataDescriptionField = {
  name: string;
  field_type: FieldType;
  mandatory: boolean;
  readonly: boolean;
  for_subtypes?: string[];
};

export type FieldType =
  | { String: EmptyObj }
  | { MarkupString: EmptyObj }
  | { Flag: EmptyObj }
  | { NaturalNumber: EmptyObj }
  | { Ref: string }
  | { RefList: string }
  | { BLOBId: EmptyObj }
  | { Enum: string[] }
  | { Date: EmptyObj }
  | { Duration: EmptyObj }
  | { People: EmptyObj }
  | { Countries: EmptyObj };

declare global {
  interface Window {
    SCHEMA: DataSchema;
  }
}

export function getDocumentTypes(collections: boolean): string[] {
  return window.SCHEMA.modules
    .filter((module) => isModuleCollection(module) === collections)
    .map((module) => module.document_type)
    .sort();
}

export function getDataDescription(documentType: string): DataDescription {
  const dataDescription = window.SCHEMA.modules.find(
    (module) => module.document_type === documentType
  );
  if (!dataDescription) {
    throw new Error(`Can't find data description for "${documentType}"`);
  }

  return dataDescription;
}

function isModuleCollection(module: DataDescription): boolean {
  return module.collection_of !== 'None';
}

export function isDocumentTypeCollection(documentType: string): boolean {
  const dataDescription = getDataDescription(documentType);

  return isModuleCollection(dataDescription);
}

export function getFieldDescriptions(
  documentType: string,
  subtype?: string
): DataDescriptionField[] {
  const dataDescription = getDataDescription(documentType);

  if (subtype === undefined) {
    return dataDescription.fields;
  }

  return dataDescription.fields.filter((field) => isFieldActive(field, subtype));
}

export function isFieldActive(field: DataDescriptionField, subtype: string): boolean {
  return field.for_subtypes?.includes(subtype) ?? true;
}

export function getDefaultSubtype(documentType: string): string {
  const dataDescription = getDataDescription(documentType);

  return dataDescription.subtypes?.[0] ?? '';
}

export function isAttachment(documentType: string) {
  return documentType === 'attachment';
}

export function isErasedDocument(documentType: string) {
  return documentType === '';
}

export function isImageAttachment(subtype: string) {
  return subtype === 'image';
}

export function isAudioAttachment(subtype: string) {
  return subtype === 'audio';
}
