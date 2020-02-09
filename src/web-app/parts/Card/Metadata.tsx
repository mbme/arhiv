import * as React from 'react'
import { Document } from '~/arhiv/replica'
import {
  Label,
  Box,
} from '~/web-platform'
import { ChronoFormatter } from '~/chrono'

const dateFormat = new ChronoFormatter('YYYY-MM-DD HH:mm')

interface IProps {
  document: Document
}

export function Metadata({ document }: IProps) {
  return (
    <>
      <Box mb="medium">
        <Label>id</Label>
        {document.id}
      </Box>

      <Box mb="medium">
        <Label>type</Label>
        {document.type}
      </Box>

      <Box mb="medium">
        <Label>revision</Label>
        {document.rev}
      </Box>

      <Box mb="medium">
        <Label>created at</Label>
        {dateFormat.format(document.createdAt)}
      </Box>

      <Box mb="medium">
        <Label>update at</Label>
        {dateFormat.format(document.updatedAt)}
      </Box>

      <Box mb="medium">
        <Label>deleted</Label>
        {document.deleted.toString()}
      </Box>
    </>
  )
}
