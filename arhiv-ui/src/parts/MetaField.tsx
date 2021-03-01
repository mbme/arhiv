import * as React from 'react'
import { Box, Label } from '@v/web-platform'

interface IProps {
  title: string
  children: React.ReactNode
}

export function MetaField({ title, children }: IProps) {
  return (
    <Box mb="medium">
      <Label>{title}</Label>
      {children}
    </Box>
  )
}
