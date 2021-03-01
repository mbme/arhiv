import * as React from 'react'
import { Box, CodeBlock, Heading } from '@v/web-platform'
import { prettyPrintJSON } from '@v/utils'

interface IProps {
  error: any
}

export function ErrorBlock({ error }: IProps) {
  return (
    <Box bgColor="red">
      <Heading variant="2">
        We've got a problem:
      </Heading>

      <CodeBlock>
        {prettyPrintJSON(error)}
      </CodeBlock>
    </Box>
  )
}
