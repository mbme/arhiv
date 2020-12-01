import * as React from 'react'
import {  RouterContext } from '@v/web-utils'
import { CardLoader, Card } from '../parts'

interface IProps {
  id: string
}

export function DocumentCardView({ id }: IProps) {
  const router = RouterContext.use()

  const onEdit = () => router.replace(`/documents/${id}/edit` )
  const onClose = () => router.goBack()

  return (
    <CardLoader id={id}>
      {document => (
        <Card
          document={document}
          onEdit={onEdit}
          onClose={onClose}
        />
      )}
    </CardLoader>
  )
}
