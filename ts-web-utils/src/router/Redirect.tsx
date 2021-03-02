import * as React from 'react'
import {
  SimpleLocation,
} from './types'
import { RouterContext } from './context'

interface IProps {
  to: SimpleLocation
  replace?: boolean
}

export function Redirect({ to, replace }: IProps) {
  const router = RouterContext.use()

  React.useEffect(() => {
    if (replace) {
      router.replace(to)
    } else {
      router.push(to)
    }
  }, [router, to, replace])

  return null
}
