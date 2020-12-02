import * as React from 'react'
import { Box, Label } from '@v/web-platform'
import { IDocumentData } from '../../api'
import { useDataDescription } from '../../data-manager'
import { CardDataField } from './CardDataField'

interface IProps {
  data: IDocumentData
}

export function CardData({ data }: IProps) {
  const {
    dataDescription,
    titleField,
  } = useDataDescription(data.type)

  const fields = Object.entries(dataDescription.fields).map(([name, field]) => {
    const value = data[name]

    return (
      <Box key={name} mb="medium">
        <Label>{name}</Label>
        <CardDataField
          value={value}
          fieldType={field.fieldType}
          isTitle={name === titleField}
        />
      </Box>
    )
  })

  return (
    <>
      {fields}
    </>
  )
}
