import * as React from 'react'
import {
  SimpleLocation,
} from './types'
import { RouterContext } from './context'

interface IProps {
  to: SimpleLocation
}

export function Redirect({ to }: IProps) {
  const router = RouterContext.use()

  React.useEffect(() => {
    router.push(to)
  }, [router, to])

  return null
}
