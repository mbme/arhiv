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
        {message.labelIds.join(',')}
        <br/>
        {message.snippet}
        <br/>
        <hr />
        {message.payload.headers.map(({name, value}) => <Box>{name}: {value}</Box>)}
        <hr />
        {message.payload.parts?.map((part) => {
          const data = part.body.data

          const value = Buffer.alloc(data.length, data, 'base64').toString()
          return (
            <Box key={part.partId}>
              {value}
            </Box>
          )
        })}
      </Box>
    )
  })

  return (
    <Box>
      {items}
    </Box>
  )
}
