import * as React from 'react'
import { Box, StyleArg } from '@v/web-platform'
import { RouterContext, useObservable } from '@v/web-utils'

const $style: StyleArg = {
  position: 'fixed',
  bottom: 0,
  left: 0,
  zIndex: 1,
  backgroundColor: 'var(--color-bg-overlay)',
}

export function Url() {
  const router = RouterContext.use()
  const [locationStr] = useObservable(() => router.locationRaw$.value$)

  return (
    <Box
      $style={$style}
    >
      {locationStr}
    </Box>
  )
}
