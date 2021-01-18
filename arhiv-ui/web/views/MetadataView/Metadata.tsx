import * as React from 'react'
import {
  Box,
} from '@v/web-platform'
import { ChronoFormatter } from '@v/chrono'
import { IDocument } from '../../api'
import { MetaField, Ref } from '../../parts'

const dateFormat = new ChronoFormatter('YYYY-MM-DD HH:mm')

interface IProps {
  document: IDocument
}

export function Metadata({ document }: IProps) {
  return (
    <>
      <MetaField title="id">
        {document.id}
      </MetaField>

      <MetaField title="type">
        {document.documentType}
      </MetaField>

      <MetaField title="revision">
        {document.rev}
      </MetaField>

      <MetaField title="created at">
        {dateFormat.format(new Date(document.createdAt))}
      </MetaField>

      <MetaField title="updated at">
        {dateFormat.format(new Date(document.updatedAt))}
      </MetaField>

      <MetaField title="archived">
        {document.archived.toString()}
      </MetaField>

      <MetaField title="refs">
        {document.refs.map(ref => (
          <Box key={ref}>
            <Ref id={ref} />
          </Box>
        ))}
      </MetaField>
    </>
  )
}
