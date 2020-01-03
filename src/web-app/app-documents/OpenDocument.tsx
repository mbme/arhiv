import * as React from 'react'
import { Box } from '~/web-platform'

interface IProps {
  id: string
}

export function OpenDocument({ id }: IProps) {
  return (
    <Box
      width="300px"
      mx="large"
      mt="large"
      minHeight="300px"
      flex="0 0 auto"
    >
      {id}
    </Box>
  )
}
