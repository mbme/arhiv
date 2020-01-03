import * as React from 'react'
import { DocumentsList } from './DocumentsList'
import { OpenDocuments } from './OpenDocuments'

interface IProps {
  ids: string[],
  filter: string,
}

export function WorkspaceView({ ids, filter }: IProps) {
  return (
    <div>
      <DocumentsList filter={filter} />
      <OpenDocuments ids={ids} />
    </div>
  )
}
