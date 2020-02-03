import * as React from 'react'
import { WorkspaceView } from './WorkspaceView'
import { WorkspaceStore, WorkspaceStoreContext } from './store'
import { RouterContext, paramAsString } from '~/web-router'

export const WorkspaceViewContainer = React.memo(function WorkspaceViewContainer() {
  const router = RouterContext.use()

  const store = new WorkspaceStore()

  const filter = paramAsString(router.location$.value.params, 'filter')
  store.updateFilter(filter)

  for (const param of router.location$.value.params) {
    if (param.name === 'filter') {
      store.updateFilter(param.value || '')
      continue
    }

    if (param.name === 'id' && param.value) {
      store.openDocument(param.value)
      continue
    }

    if (param.name === 'create' && param.value) {
      store.createDocument(param.value)
      continue
    }
  }

  React.useEffect(() => store.state$.subscribe({
    next(state) {
      router.replaceParams([
        {
          name: 'filter',
          value: state.filter || undefined,
        },
        ...state.items.map((item) => {
          if (item._type === 'document') {
            return {
              name: 'id',
              value: item.id,
            }
          }

          if (item._type === 'new-document') {
            return {
              name: 'create',
              value: item.type,
            }
          }

          throw new Error('unexpected workspace item type')
        }).reverse(),
      ])
    },
  }), [store, router])

  return (
    <WorkspaceStoreContext.Provider value={store}>
      <WorkspaceView />
    </WorkspaceStoreContext.Provider>
  )
})
