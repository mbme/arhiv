import * as React from 'react'
import {
  Location,
  useRouter,
} from './Router'

interface IProps {
  to: Location
}

export function Redirect({ to }: IProps) {
  const router = useRouter()
  React.useEffect(() => {
    router.push(to)
  }, [])

  return null
}
