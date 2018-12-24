type Handler<T, K extends keyof T> = (params: T[K]) => void

export default class PubSub<T> {
  subs = new Map()

  getEventSubs<K extends keyof T>(name: K): Set<Handler<T, K>> {
    return this.subs.get(name) || new Set()
  }

  on<K extends keyof T>(name: K, handler: Handler<T, K>) {
    const eventSubs = this.getEventSubs(name)
    eventSubs.add(handler)

    this.subs.set(name, eventSubs)
  }

  off<K extends keyof T>(name: K, handler: Handler<T, K>) {
    const eventSubs = this.getEventSubs(name)
    eventSubs.delete(handler)

    if (!eventSubs.size) {
      this.subs.delete(name)
    }
  }

  emit<K extends keyof T>(name: K, params: T[K]) {
    this.getEventSubs<K>(name).forEach(handler => handler(params))
  }
}
