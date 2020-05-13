import * as React from 'react'
import { Box } from '@v/web-platform'
import { usePromise } from '@v/web-utils'
import { list } from './notes'

export function App() {
  const [notes] = usePromise(() => list(), []);

  if (!notes) {
    return null
  }

  return (
    <Box
      width="50rem"
      m="0 auto"
      p="large"
    >
      <h1>Hello world</h1>
      {notes.map(note => (
        <Box>
          {JSON.stringify(note, null, 2)}
        </Box>
      ))}
    </Box>
  )
}
