const pad = (s: string, maxLength: number) => s.padStart(maxLength, '0')

const YYYY = (date: Date) => date.getFullYear().toString()
const MM = (date: Date) => (date.getMonth() + 1).toString()
const DD = (date: Date) => pad(date.getDate().toString(), 2)
const HH = (date: Date) => pad(date.getHours().toString(), 2)
const mm = (date: Date) => pad(date.getMinutes().toString(), 2)
const ss = (date: Date) => pad(date.getSeconds().toString(), 2)
const SSS = (date: Date) => pad(date.getSeconds().toString(), 3)

export function formatDate(date: Date) {
  return `${YYYY(date)}-${MM(date)}-${DD(date)} ${HH(date)}:${mm(date)}:${ss(date)},${SSS(date)}`
}

export function formatTs(ts: number) {
  const date = new Date(ts * 1000)

  return [
    date.getFullYear(),
    date.getMonth() + 1,
    date.getDate(),
  ].join('/')
}

export const nowS = () => Date.now() / 1000

export function dateNow(): string { // ISO8601
  return new Date().toISOString()
}
