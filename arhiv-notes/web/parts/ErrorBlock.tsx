import * as React from 'react'
import { Box, Heading } from '@v/web-platform'
import { prettyPrintJSON } from '@v/utils'

interface IProps {
  error: any
}

export function ErrorBlock({ error }: IProps) {
  return (
    <Box bgColor="red">
      <Heading>We've got a problem</Heading>

      <pre>
        <code>
          {prettyPrintJSON(error)}
        </code>
      </pre>
    </Box>
  )
}
