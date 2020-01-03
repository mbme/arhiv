import * as React from 'react'
import { Row } from '~/web-platform'
import { DocumentsList } from './DocumentsList'
import { OpenDocuments } from './OpenDocuments'

interface IProps {
  ids: string[],
  filter: string,
}

export function WorkspaceView({ ids, filter }: IProps) {
  return (
    <Row
      alignX="left"
      alignY="top"
      overflowX="scroll"
    >
      <DocumentsList filter={filter} />

      <OpenDocuments ids={ids} />
    </Row>
  )
}
