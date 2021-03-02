import * as React from 'react'
import { Input, Select, Textarea } from '@v/web-platform'
import { Dict } from '@v/utils'
import { FieldType } from '../../../api'

interface IProps {
  name: string
  fieldType: FieldType
}

export function CardEditorFormField({ name, fieldType }: IProps) {
  if ('String' in fieldType) {
    return (
      <Input
        label={name}
        name={name}
        placeholder={name}
        autoComplete="off"
      />
    )
  }

  if ('MarkupString' in fieldType) {
    return (
      <Textarea
        label={name}
        name={name}
        placeholder={name}
      />
    )
  }

  if ('Ref' in fieldType) {
    return (
      <Input
        label={name}
        name={name}
        placeholder={name}
      />
    )
  }

  if ('Enum' in fieldType) {
    const options: Dict = {}
    for (const value of fieldType.Enum) {
      options[value] = value
    }

    return (
      <Select
        label={name}
        name={name}
        options={options}
      />
    )
  }

  throw new Error(`Unsupported field type ${JSON.stringify(fieldType)}`)
}
