import { EmptyObj } from '../scripts/utils';

export type DataSchema = {
  modules: DataDescription[];
};

export type DataDescription = {
  document_type: string;
  subtypes?: string[];
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

export function getDataDescription(documentType: string): DataDescription {
  const dataDescription = window.SCHEMA.modules.find(
    (module) => module.document_type === documentType
  );
  if (!dataDescription) {
    throw new Error(`Can't find data description for "${documentType}"`);
  }

  return dataDescription;
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
