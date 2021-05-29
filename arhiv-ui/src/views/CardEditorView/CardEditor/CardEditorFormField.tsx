import * as React from 'react'
import { Input, Select, Textarea } from '@v/web-platform'
import { Dict } from '@v/utils'
import { IField } from '@v/arhiv-api'

interface IProps {
  field: IField
}

export function CardEditorFormField({ field: { name, fieldType, optional } }: IProps) {
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
        label={`${name} (Ref to ${fieldType.Ref})`}
        name={name}
        placeholder={name}
      />
    )
  }

  if ('Enum' in fieldType) {
    const options: Dict<string | undefined> = {}

    if (optional) {
      options[''] = undefined
    }

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

  return (
    <Input
      label={name}
      name={name}
      placeholder={name}
      autoComplete="off"
    />
  )
}
