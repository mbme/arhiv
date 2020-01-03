import * as React from 'react'
import { IApp } from '../chrome'
import { WorkspaceView } from './WorkspaceView'

function parseIds(ids: string): string[] {
  if (!ids.length) {
    return []
  }

  return ids.split(',')
}

export const DocumentsApp: IApp = {
  name: 'Documents',
  route: '/',
  render({ ids, filter }) {
    return (
      <WorkspaceView ids={parseIds(ids || '')} filter={filter || ''} />
    )
  },
}
