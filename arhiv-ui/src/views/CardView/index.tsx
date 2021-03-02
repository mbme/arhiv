import * as React from 'react'
import {
  CardLoader,
  CatalogOptionsOverrides,
  FrameTitle,
} from '../../parts'
import { Card } from './Card'

interface IProps {
  id: string
  catalogOptions?: CatalogOptionsOverrides
}

export function CardView({ id, catalogOptions }: IProps) {
  return (
    <CardLoader id={id}>
      {document => (
        <>
          <FrameTitle>
            {document.documentType} Card
          </FrameTitle>

          <Card
            document={document}
            catalogOptions={catalogOptions}
          />
        </>
      )}
    </CardLoader>
  )
}
