export * from './api'
export * from './types'

export const createLink = (url: string, text = '') => (
  text ? `[[${url}][${text}]]` : `<${url}>`
)

export const createRef = (ref: string) => createLink(`ref:${ref}`)
