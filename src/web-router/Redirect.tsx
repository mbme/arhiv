import * as React from 'react'
import {
  SimpleLocation,
} from './types'
import { useRouter } from './Router'

interface IProps {
  to: SimpleLocation
}

export function Redirect({ to }: IProps) {
  const router = useRouter()

  React.useEffect(() => {
    router.push(to)
  }, [router, to])

  return null
}
