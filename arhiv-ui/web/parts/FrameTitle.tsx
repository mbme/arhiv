import * as React from 'react'
import { Heading } from '@v/web-platform'

interface IProps {
  children: React.ReactNode
}

export function FrameTitle({ children }: IProps) {
  return (
    <Heading
      fontSize="medium"
      uppercase
      color="var(--color-secondary)"
    >
      {children}
    </Heading>
  )
}
