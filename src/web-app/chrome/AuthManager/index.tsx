import * as React from 'react'
import { useObservable } from '~/reactive/react'
import { Arhiv } from '~/arhiv'
import { AuthOverlay } from './AuthOverlay'

interface IProps {
  arhiv: Arhiv
}

export function AuthManager({ arhiv }: IProps) {
  const [authorized, isReady] = useObservable(() => arhiv.isAuthorized$.value$)

  if (authorized || !isReady) {
    return null
  }

  return (
    <AuthOverlay submit={password => arhiv.authorize(password)} />
  )
}
