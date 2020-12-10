import * as React from 'react'
import {  RouterContext } from '@v/web-utils'
import { Action, CardLoader, Frame } from '../../parts'
import { Metadata } from './Metadata'

interface IProps {
  id: string
}

export function MetadataView({ id }: IProps) {
  const router = RouterContext.use()

  const onEdit = () => router.replace(`/documents/${id}/edit` )
  const onShow = () => router.replace(`/documents/${id}`)

  const actions = (
    <>
      <Action
        type="action"
        onClick={onShow}
      >
        Show Document
      </Action>

      <Action
        type="action"
        onClick={onEdit}
      >
        Edit Document
      </Action>
    </>
  )

  return (
    <Frame
      actions={actions}
      title="Metadata"
    >
      <CardLoader id={id}>
        {document => (
          <Metadata document={document} />
        )}
      </CardLoader>
    </Frame>
  )
}
