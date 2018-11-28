export default function createPubSub<T>() {
  type Handler = (params?: T) => void

  const subs = new Map<string, Set<Handler>>()

  const getEventSubs = (name: string) => (subs.get(name) || new Set())

  return {
    on(name: string, handler: Handler) {
      const eventSubs = getEventSubs(name)
      eventSubs.add(handler)

      subs.set(name, eventSubs)
    },

    off(name: string, handler: Handler) {
      const eventSubs = getEventSubs(name)
      eventSubs.delete(handler)

      if (!eventSubs.size) {
        subs.delete(name)
      }
    },

    emit(name: string, params?: T) {
      getEventSubs(name).forEach(handler => handler(params))
    },
  }
}
