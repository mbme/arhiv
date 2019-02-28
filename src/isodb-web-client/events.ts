import PubSub from '~/utils/pubsub'
import { IEvents as IDBEvents } from './replica'

interface IEvents extends IDBEvents {
  'authorized': boolean
  'network-online': boolean
  'network-error': number
  'isodb-lock': [boolean, Set<string>]
}

export type WebClientEvents = PubSub<IEvents>

export function createEventsPubSub() {
  return new PubSub<IEvents>()
}
