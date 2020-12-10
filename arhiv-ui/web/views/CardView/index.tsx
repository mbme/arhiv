import * as React from 'react'
import {  RouterContext } from '@v/web-utils'
import { CardLoader } from '../../parts'
import { Card } from './Card'

interface IProps {
  id: string
}

export function CardView({ id }: IProps) {
  const router = RouterContext.use()

  const onEdit = () => router.replace(`/documents/${id}/edit` )
  const onClose = () => router.goBack()
  const onMetadata = () => router.replace(`/documents/${id}/metadata` )

  return (
    <CardLoader id={id}>
      {document => (
        <Card
          document={document}
          onEdit={onEdit}
          onClose={onClose}
          onMetadata={onMetadata}
        />
      )}
    </CardLoader>
  )
}
