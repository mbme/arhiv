type Event<T> = keyof T
type Params<T> = T[Event<T>]
type Handler<T> = (params: Params<T>) => void
type Handlers<T> = Set<Handler<T>>
type HandlersMap<T> = Map<Event<T>, Handlers<T>>

export default class PubSub<T> {
  subs: HandlersMap<T> = new Map()

  getEventSubs(name: Event<T>) {
    return this.subs.get(name) || new Set()
  }

  on(name: Event<T>, handler: Handler<T>) {
    const eventSubs = this.getEventSubs(name)
    eventSubs.add(handler)

    this.subs.set(name, eventSubs)
  }

  off(name: Event<T>, handler: Handler<T>) {
    const eventSubs = this.getEventSubs(name)
    eventSubs.delete(handler)

    if (!eventSubs.size) {
      this.subs.delete(name)
    }
  }

  emit(name: Event<T>, params: Params<T>) {
    this.getEventSubs(name).forEach(handler => handler(params))
  }
}
