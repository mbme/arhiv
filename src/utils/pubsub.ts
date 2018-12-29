type Handler<T, K extends keyof T> = (params: T[K]) => void

export default class PubSub<T> {
  private readonly subs = new Map()

  on<K extends keyof T>(name: K, handler: Handler<T, K>) {
    const eventSubs = this._getEventSubs(name)
    eventSubs.add(handler)

    this.subs.set(name, eventSubs)
  }

  off<K extends keyof T>(name: K, handler: Handler<T, K>) {
    const eventSubs = this._getEventSubs(name)
    eventSubs.delete(handler)

    if (!eventSubs.size) {
      this.subs.delete(name)
    }
  }

  emit<K extends keyof T>(name: K, params: T[K]) {
    this._getEventSubs(name).forEach((handler: Handler<T, K>) => handler(params))
  }

  private _getEventSubs(name: string): Set<any> {
    return this.subs.get(name) as Set<any> || new Set()
  }
}
