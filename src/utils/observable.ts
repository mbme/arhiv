import { removeMut } from './index'

export default function observable<T>(initialValue: T) {
  type Sub = (value: T) => void

  const subs: Sub[] = []
  let value = initialValue

  return {
    get value() {
      return value
    },

    set(newValue: T) {
      value = newValue
      subs.forEach((sub) => sub(newValue))
    },

    on(sub: Sub) {
      subs.push(sub)

      return () => removeMut(subs, sub)
    },
  }
}
