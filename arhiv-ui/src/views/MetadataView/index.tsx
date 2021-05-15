import * as React from 'react'
import {  RouterContext } from '@v/web-platform'
import { CardLoader, FrameTitle, useActions } from '../../parts'
import { Metadata } from './Metadata'

interface IProps {
  id: string
}

export function MetadataView({ id }: IProps) {
  const router = RouterContext.use()

  const onEdit = () => router.replace(`/documents/${id}/edit` )
  const onClose = () => router.replace(`/documents/${id}`)

  useActions(() => [
    {
      onClick: onClose,
      children: 'Close',
    },
    {
      onClick: onEdit,
      children: 'Edit Document',
    },
  ], [])

  return (
    <CardLoader id={id}>
      {document => (
        <>
          <FrameTitle>
            {document.documentType} Metadata
          </FrameTitle>

          <Metadata document={document} />
        </>
      )}
    </CardLoader>
  )
}
