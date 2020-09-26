import * as React from 'react'
import {
  Box,
  StyleArg,
  useFocusedRegion,
} from '@v/web-platform'

interface IProps {
  children: React.ReactNode
  $style?: StyleArg
}

export function Content({ children, $style }: IProps) {
  const isActive = useFocusedRegion()

  return (
    <Box
      px="medium"
      width="40rem"
      overflowY="auto"
      borderLeft="default"
      borderRight="default"
      bgColor={isActive ? 'var(--color-bg-highlight)' : undefined}
      $style={$style}
    >
      {children}
    </Box>
  )
}
