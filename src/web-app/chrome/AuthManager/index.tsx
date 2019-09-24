import * as React from 'react'
import { useReactiveValue } from '~/utils/react'
import { Arhiv } from '~/arhiv'
import { AuthOverlay } from './AuthOverlay'

interface IProps {
  arhiv: Arhiv
}

export function AuthManager({ arhiv }: IProps) {
  const authorized = useReactiveValue(() => arhiv.net.authorized$)

  if (authorized) {
    return null
  }

  return (
    <AuthOverlay submit={arhiv.net.authorize} />
  )
}
