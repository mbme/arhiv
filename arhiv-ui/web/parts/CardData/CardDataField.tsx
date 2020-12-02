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
  if (fieldType === 'String' && isTitle) {
    return (
      <Heading
        fontSize="large"
      >
        {value}
      </Heading>
    )
  }

  if (fieldType === 'MarkupString') {
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
