import * as React from 'react'
import { Box } from '@v/web-platform'
import { PoshtaStore } from './poshta-store'
import { useObservable } from '@v/web-utils'

interface IProps {
  store: PoshtaStore,
}

export function App({ store }: IProps) {
  const [state] = useObservable(() => store.state$)

  if (!state) {
    return null
  }

  const items = state.messages.map((message) => {
    console.error(message)
    return (
      <Box key={message.id} mb="large">
        <hr />
        {message.labelIds.join(', ')}
        <br/>
        <h2 dangerouslySetInnerHTML={{ __html: message.snippet }} />
        <br/>
        {message.payload.headers.map(({name, value}) => <Box><b>{name}</b>: {value}</Box>)}
      </Box>
    )
  })

  return (
    <Box>
      {items}
    </Box>
  )
}
