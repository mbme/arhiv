import { Callbacks } from '~/utils/callbacks'
import { ISubscription } from './types'

export function createSubscription(): ISubscription {
  const callbacks = new Callbacks()

  const subscription: ISubscription = () => callbacks.runAll(true)
  subscription.callbacks = callbacks

  return subscription
}
