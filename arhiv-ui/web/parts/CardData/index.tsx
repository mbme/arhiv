import * as React from 'react'
import { Box, Label, Spacer } from '@v/web-platform'
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

  const fields = dataDescription.fields.map(({ name, fieldType }) => {
    const value = data[name]

    return (
      <Box key={name} mb="medium">
        <Label>
          {name}
        </Label>
        <Spacer height="small" />
        <CardDataField
          value={value}
          fieldType={fieldType}
          isTitle={name === titleField}
        />
      </Box>
    )
  })

  return (
    <Box pr="medium">
      {fields}
    </Box>
  )
}
