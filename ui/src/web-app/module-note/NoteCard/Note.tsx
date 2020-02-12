import * as React from 'react'
import {
  Heading,
} from '~/web-platform'
import {
  Markup,
} from '~/web-app/parts'

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
