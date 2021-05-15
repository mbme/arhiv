import { parseInt10 } from '@v/utils'

export function getTabId(propName: string): number {
  const tabIdStr = sessionStorage.getItem(propName)
  if (tabIdStr) {
    return parseInt10(tabIdStr)
  }

  const lastTabId = parseInt10(localStorage.getItem(propName) || '0')
  const tabId = lastTabId + 1

  localStorage.setItem(propName, tabId.toString())
  sessionStorage.setItem(propName, tabId.toString())

  return tabId
}
