import * as React from 'react'
import { CardLoader } from '../../parts'
import { Card } from './Card'

interface IProps {
  id: string
}

export function CardView({ id }: IProps) {
  return (
    <CardLoader id={id}>
      {document => (
        <Card
          document={document}
        />
      )}
    </CardLoader>
  )
}
