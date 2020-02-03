import * as React from 'react'
import {
  Column,
} from '~/web-platform'
import { CardContainer } from './CardContainer'
import { useWorkspaceStore } from '../store'

export function OpenCards() {
  const store = useWorkspaceStore()
  const items = store.state.items.map((item) => {
    if (item._type === 'document') {
      return (
        <CardContainer
          key={item.id}
          item={item}
          focused={store.state.focused === item}
        />
      )
    }

    return (
      <CardContainer
        key={item.tempId}
        item={item}
        focused={store.state.focused === item}
      />
    )
  })

  return (
    <Column
      alignX="center"
    >
      {items}
    </Column>
  )
}
