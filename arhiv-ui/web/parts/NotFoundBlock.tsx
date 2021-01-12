import * as React from 'react'
import { Box, Heading } from '@v/web-platform'

interface IProps {
  children: React.ReactNode
}

export function NotFoundBlock({ children }: IProps) {
  return (
    <Box bgColor="red">
      <Heading variant="2">
        Not Found
      </Heading>

      <Box p="medium">
        {children}
      </Box>
    </Box>
  )
}
