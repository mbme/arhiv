import * as React from 'react'
import {
  Markup,
} from '../parts'

interface IProps {
  name: string
  data: string
}

export function Note({ name, data }: IProps) {
  return (
    <>
      <h1>
        {name}
      </h1>

      <Markup value={data} />
    </>
  )
}
