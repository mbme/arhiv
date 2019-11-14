import * as React from 'react'
import { useArhiv } from '~/arhiv'
import { useObservable } from '~/web-platform'
import { AuthOverlay } from './AuthOverlay'

export function AuthManager() {
  const arhiv = useArhiv()
  const [authorized, isReady] = useObservable(() => arhiv.isAuthorized$.value$)

  if (authorized || !isReady) {
    return null
  }

  return (
    <AuthOverlay submit={password => arhiv.authorize(password)} />
  )
}
