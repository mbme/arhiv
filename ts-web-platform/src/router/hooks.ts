import { useCell } from '../utils'
import { RouterContext } from './context'
import { ILocation } from './types'

export function useLocation(): ILocation {
  const router = RouterContext.use()
  const [location] = useCell(router.location$)

  return location
}
