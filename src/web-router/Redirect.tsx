import * as React from 'react'
import { SimpleLocation } from './web-router'
import { useRouter } from './Router'

interface IProps {
  to: SimpleLocation
}

export function Redirect({ to }: IProps) {
  const router = useRouter()
  React.useEffect(() => {
    router.push(to)
  }, [])

  return null
}
