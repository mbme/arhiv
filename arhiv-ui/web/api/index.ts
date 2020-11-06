export * from './api'
export * from './types'

export * from './note'
export * from './project'
export * from './task'

export const createLink = (url: string, text = '') => (
  text ? `[[${url}][${text}]]` : `[[${url}]]`
)
