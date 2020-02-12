import * as React from 'react'
import {
  Column,
} from '~/web-platform'
import { CardContainer } from './CardContainer'
import { useWorkspaceStore } from '../../workspace-store'

export function OpenCards() {
  const store = useWorkspaceStore()
  const items = store.state.items.map(document => (
    <CardContainer
      key={document.id}
      document={document}
      focused={store.state.focusedId === document.id}
    />
  ))

  return (
    <Column
      alignX="center"
    >
      {items}
    </Column>
  )
}
