import { Heading } from '@v/web-platform'
import * as React from 'react'
import { FieldType } from '../../api'
import { Markup } from '../Markup'

interface IProps {
  value: any
  fieldType: FieldType
  isTitle: boolean
}

export function CardDataField({ value, fieldType, isTitle }: IProps) {
  if ('String' in fieldType && isTitle) {
    return (
      <Heading
        variant="2"
      >
        {value}
      </Heading>
    )
  }

  if ('MarkupString' in fieldType) {
    return  (
      <Markup value={value} />
    )
  }

  return (
    <>
      {value}
    </>
  )
}
