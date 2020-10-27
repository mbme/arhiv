import * as React from 'react'
import {
  Label,
  Box,
} from '@v/web-platform'
import { ChronoFormatter } from '@v/chrono'
import { Note } from './api'

const dateFormat = new ChronoFormatter('YYYY-MM-DD HH:mm')

interface IProps {
  document: Note
}

// FIXME render refs and attachment refs
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
        {dateFormat.format(new Date(document.createdAt))}
      </Box>

      <Box mb="medium">
        <Label>update at</Label>
        {dateFormat.format(new Date(document.updatedAt))}
      </Box>

      <Box mb="medium">
        <Label>archived</Label>
        {document.archived.toString()}
      </Box>
    </>
  )
}
