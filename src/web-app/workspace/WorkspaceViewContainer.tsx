import * as React from 'react'
import { WorkspaceView } from './WorkspaceView'
import { ArhivContext } from '../arhiv-context'
import { WorkspaceStore, WorkspaceStoreContext } from '../workspace-store'

export const WorkspaceViewContainer = React.memo(function WorkspaceViewContainer() {
  const arhiv = ArhivContext.use()
  const store = new WorkspaceStore(arhiv)

  return (
    <WorkspaceStoreContext.Provider value={store}>
      <WorkspaceView />
    </WorkspaceStoreContext.Provider>
  )
})
