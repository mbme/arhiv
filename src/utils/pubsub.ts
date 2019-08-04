import { TypeOfProperty } from './types'

interface IEvent {
  name: string
}

type Handler<T = any> = (event: T) => void

export class PubSub<T extends IEvent> {
  private _subs = new Map<string, Set<Handler>>()

  on<K extends T>(name: TypeOfProperty<K, 'name'>, handler: Handler<K>) {
    const eventSubs = this._getEventSubs(name)
    eventSubs.add(handler)

    this._subs.set(name, eventSubs)
  }

  off<K extends T>(name: TypeOfProperty<K, 'name'>, handler: Handler<K>) {
    const eventSubs = this._getEventSubs(name)
    eventSubs.delete(handler)

    if (!eventSubs.size) {
      this._subs.delete(name)
    }
  }

  emit(event: T) {
    for (const handler of this._getEventSubs(event.name)) {
      handler(event)
    }
  }

  private _getEventSubs(name: string): Set<Handler> {
    return this._subs.get(name) || new Set()
  }

  destroy() {
    this._subs.clear()
  }
}
