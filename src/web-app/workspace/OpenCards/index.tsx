import * as React from 'react'
import {
  Column,
} from '~/web-platform'
import { CardContainer } from './CardContainer'
import { useWorkspaceURLManager } from '../useWorkspaceURLManager'

interface IProps {
  newestId: string | undefined
}

export function OpenCards({ newestId }: IProps) {
  const ws = useWorkspaceURLManager()

  return (
    <Column
      alignX="center"
    >
      {ws.openIds.map(id => (
        <CardContainer
          key={id}
          id={id}
          focused={newestId === id}
        />
      ))}
    </Column>
  )
}
