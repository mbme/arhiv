import * as React from 'react'
import { Input, Textarea } from '@v/web-platform'
import { FieldType } from '../../api'

interface IProps {
  name: string
  fieldType: FieldType
}

export function CardEditorFormField({ name, fieldType }: IProps) {
  if (fieldType === 'String') {
    return (
      <Input
        label={name}
        name={name}
        placeholder={name}
      />
    )
  }

  if (fieldType === 'MarkupString') {
    return (
      <Textarea
        label={name}
        name={name}
        placeholder={name}
      />
    )
  }

  throw new Error(`Unsupported field type ${fieldType}`)
}
