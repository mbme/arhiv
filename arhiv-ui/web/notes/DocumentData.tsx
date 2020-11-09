import { Box, Label } from '@v/web-platform'
import * as React from 'react'
import { IDocument } from '../api'
import { IDocumentDataDescription } from '../data-description'
import { Markup } from '../Markup'

interface IProps {
  document: IDocument
  dataDescription: IDocumentDataDescription
}

export function DocumentData({ document, dataDescription }: IProps) {
  const fields = Object.entries(dataDescription).map(([name, fieldType]) => {
    const value = document.data[name]

    let field
    switch (fieldType.type) {
      case 'markup-string': {
        field = (
          <Markup value={value} />
        )
        break
      }
      default: {
        field = value
        break
      }
    }

    return (
      <Box key={name} mb="medium">
        <Label>{name}</Label>
        {field}
      </Box>
    )
  })

  return (
    <>
      {fields}
    </>
  )
}
