import * as React from 'react'
import { useObservable } from '~/utils/react'
import { Arhiv } from '~/arhiv'
import { AuthOverlay } from './AuthOverlay'

interface IProps {
  arhiv: Arhiv
}

export function AuthManager({ arhiv }: IProps) {
  const authorized = useObservable(() => arhiv.isAuthorized$.value$)

  if (authorized) {
    return null
  }

  return (
    <AuthOverlay submit={password => arhiv.authorize(password)} />
  )
}
