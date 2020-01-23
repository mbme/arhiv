import * as React from 'react'
import {
  Column,
} from '~/web-platform'
import { CardContainer } from './CardContainer'

interface IProps {
  openIds: readonly string[]
  newestId: string | undefined
}

export function OpenCards({ openIds, newestId }: IProps) {
  return (
    <Column
      alignX="center"
    >
      {openIds.map(id => (
        <CardContainer
          key={id}
          id={id}
          focused={newestId === id}
        />
      ))}
    </Column>
  )
}
