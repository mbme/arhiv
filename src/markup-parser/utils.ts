export const createLink = (url: string, text: string = '') => text ? `[[${url}][${text}]]` : `[[${url}]]`
