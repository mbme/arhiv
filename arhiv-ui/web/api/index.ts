export * from './api'
export * from './note'

export const createLink = (url: string, text = '') => (
  text ? `[[${url}][${text}]]` : `[[${url}]]`
)
