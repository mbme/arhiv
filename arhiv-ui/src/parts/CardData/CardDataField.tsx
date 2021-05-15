import * as React from 'react'
import { FieldType } from '@v/arhiv-api'
import { Markup } from '../Markup'
import { Ref } from '../Ref'

interface IProps {
  value: any
  fieldType: FieldType
}

export function CardDataField({ value, fieldType }: IProps) {
  if ('MarkupString' in fieldType) {
    return  (
      <Markup value={value} />
    )
  }

  if ('Ref' in fieldType) {
    return (
      <Ref id={value} />
    )
  }

  return (
    <>
      {value}
    </>
  )
}
