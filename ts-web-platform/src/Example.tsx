import * as React from 'react'

import { Box } from './Box'
import { Heading } from './Heading'

interface IProps {
  section?: boolean
  title: string
  children: React.ReactNode
}

export function Example({ section, title, children }: IProps) {
  return (
    <Box mb={section ? 'xlarge' : 'medium'}>
      {title && (
        <Heading fontSize={section ? 'large' : 'medium'}>
          {title}
        </Heading>
      )}

      {children}
    </Box>
  )
}
