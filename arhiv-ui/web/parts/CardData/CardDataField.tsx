import { Heading } from '@v/web-platform'
import * as React from 'react'
import { FieldType, IAttachmentSource } from '../../api'
import { Markup } from '../Markup'

interface IProps {
  value: any
  fieldType: FieldType
  isTitle: boolean
  newAttachments: IAttachmentSource[]
}

export function CardDataField({ value, fieldType, isTitle, newAttachments }: IProps) {
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
      <Markup value={value} newAttachments={newAttachments} />
    )
  }

  return (
    <>
      {value}
    </>
  )
}
