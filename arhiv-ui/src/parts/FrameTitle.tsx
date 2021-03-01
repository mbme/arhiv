import * as React from 'react'
import { Text } from '@v/web-platform'

interface IProps {
  children: React.ReactNode
}

export function FrameTitle({ children }: IProps) {
  return (
    <Text
      as="h4"
      fontSize="medium"
      uppercase
      color="var(--color-secondary)"
    >
      {children}
    </Text>
  )
}
