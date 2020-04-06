import * as React from 'react'
import { Box } from '@v/web-platform'
import { useObservable } from '@v/web-utils'
import { PoshtaStore } from './poshta-store'
import { MessageShort } from './MessageShort'

interface IProps {
  store: PoshtaStore,
}

export function App({ store }: IProps) {
  const [state] = useObservable(() => store.state$)

  if (!state) {
    return null
  }

  const items = state.messages.map(message => (
    <MessageShort key={message.id} message={message} />
  ))

  return (
    <Box
      maxWidth="50rem"
      m="0 auto"
      p="large"
    >
      {items}
    </Box>
  )
}
