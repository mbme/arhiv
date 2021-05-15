import { createLogger } from '@v/logger'

const log = createLogger('clipboard')

// based on https://stackoverflow.com/a/30810322
export function copyTextToClipboard(text: string): boolean {
  const textArea = document.createElement('textarea')
  textArea.value = text

  // Avoid scrolling to bottom
  textArea.style.top = '0'
  textArea.style.left = '0'
  textArea.style.position = 'fixed'

  document.body.appendChild(textArea)
  textArea.focus()
  textArea.select()

  let success = false
  try {
    success = document.execCommand('copy')
    if (!success) {
      log.error('Failed to copy text')
    }
  } catch (err) {
    log.error('Failed to copy text', err)
  }

  document.body.removeChild(textArea)

  return success
}
