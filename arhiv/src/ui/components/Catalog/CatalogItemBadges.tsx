/* eslint-disable @typescript-eslint/no-base-to-string */
import {
  DocumentType,
  DocumentData,
  BOOK_DOCUMENT_TYPE,
  FILM_DOCUMENT_TYPE,
  GAME_DOCUMENT_TYPE,
  TASK_DOCUMENT_TYPE,
  PROJECT_DOCUMENT_TYPE,
} from 'dto';
import { JSXChildren } from 'utils/jsx';

type CatalogItemBadgeProps = {
  children?: JSXChildren;
};
function CatalogItemBadge({ children }: CatalogItemBadgeProps) {
  if (!children) {
    return null;
  }

  return <div className="text-xs text-gray-400">{children}</div>;
}

type Props = {
  documentType: DocumentType;
  data: DocumentData;
};

export function CatalogItemBadges({ documentType, data }: Props) {
  if (documentType === BOOK_DOCUMENT_TYPE) {
    return (
      <>
        <CatalogItemBadge>{data['status']?.toString()}</CatalogItemBadge>
        <CatalogItemBadge>{data['rating']?.toString()}</CatalogItemBadge>
      </>
    );
  }

  if (documentType === FILM_DOCUMENT_TYPE) {
    return (
      <>
        <CatalogItemBadge>{data['status']?.toString()}</CatalogItemBadge>
        <CatalogItemBadge>{data['rating']?.toString()}</CatalogItemBadge>
      </>
    );
  }

  if (documentType === GAME_DOCUMENT_TYPE) {
    return (
      <>
        <CatalogItemBadge>{data['status']?.toString()}</CatalogItemBadge>
        <CatalogItemBadge>{data['rating']?.toString()}</CatalogItemBadge>
      </>
    );
  }

  if (documentType === TASK_DOCUMENT_TYPE) {
    return (
      <>
        <CatalogItemBadge>{data['status']?.toString()}</CatalogItemBadge>
      </>
    );
  }

  if (documentType === PROJECT_DOCUMENT_TYPE) {
    return (
      <>
        <CatalogItemBadge>{(data['tasks'] as unknown[]).length} tasks</CatalogItemBadge>
      </>
    );
  }

  return null;
}
