import * as React from 'react'
import { CardLoader, FrameTitle } from '../../parts'
import { Card } from './Card'

interface IProps {
  id: string
}

export function CardView({ id }: IProps) {
  return (
    <CardLoader id={id}>
      {document => (
        <>
          <FrameTitle>
            {document.data.type} Card
          </FrameTitle>

          <Card
            document={document}
          />
        </>
      )}
    </CardLoader>
  )
}
