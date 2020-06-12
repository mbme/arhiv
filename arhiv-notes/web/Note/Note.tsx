import * as React from 'react'
import {
  Heading,
} from '@v/web-platform'
import {
  Markup,
} from './Markup'

interface IProps {
  name: string
  data: string
}

export function Note({ name, data }: IProps) {
  return (
    <>
      <Heading
        letterSpacing="1.4px"
        fontSize="large"
      >
        {name}
      </Heading>

      <Markup value={data} />
    </>
  )
}
