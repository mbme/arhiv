import * as React from 'react'
import { Obj } from '@v/utils'
import { Box, Heading, Label } from '@v/web-platform'
import { Markup } from './Markup'
import { IDataDescription } from '../api'

interface IProps<P extends Obj> {
  data: P
  dataDescription: IDataDescription
}

export function CardData<P extends Obj>({ data, dataDescription }: IProps<P>) {
  const fields = Object.entries(dataDescription.fields).map(([name, fieldType]) => {
    const value = data[name]

    let field
    switch (fieldType.type) {
      case 'markup-string': {
        field = (
          <Markup value={value} />
        )
        break
      }

      case 'string': {
        if (fieldType.title) {
          field = (
            <Heading
              fontSize="large"
            >
              {value}
            </Heading>
          )
        } else {
          field = value
        }
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
