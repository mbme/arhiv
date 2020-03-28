import * as React from 'react'
import { isFunction } from '@v/utils'

import { Box } from './Box'
import { Heading } from './Heading'

interface IProps {
  title: string
  examples: { [description: string]: JSX.Element | React.FC }
}

export function Examples({ title, examples }: IProps) {
  return (
    <Box mb="xlarge">
      <Heading>{title}</Heading>

      {Object.entries(examples).map(([description, Example], i) => (
        <Box key={i} mb="medium">
          {description && (
            <Heading fontSize="medium">
              {description}
            </Heading>
          )}

          {isFunction(Example) ? <Example /> : Example}
        </Box>
      ))}
    </Box>
  )
}
