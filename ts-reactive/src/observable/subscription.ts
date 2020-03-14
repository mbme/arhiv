import { Callbacks } from '@v/utils'
import { ISubscription } from './types'

export function createSubscription(): ISubscription {
  const callbacks = new Callbacks()

  let active = true
  const subscription: ISubscription = () => {
    if (!active) {
      throw new Error('subscription not active')
    }

    active = false
    callbacks.runAll(true)
  }
  subscription.callbacks = callbacks

  return subscription
}
