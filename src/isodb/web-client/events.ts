import PubSub from '../../utils/pubsub'

interface IEvents {
  'authorized': boolean
  'network-online': boolean
  'network-error': number
  'isodb-lock': [boolean, Set<string>]
}

export type WebClientEvents = PubSub<IEvents>

export function createEventsPubSub() {
  return new PubSub<IEvents>()
}
