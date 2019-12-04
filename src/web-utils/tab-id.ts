export function getTabId(propName: string): number {
  const tabIdStr = sessionStorage.getItem(propName)
  if (tabIdStr) {
    return parseInt(tabIdStr, 10)
  }

  const lastTabId = parseInt(localStorage.getItem(propName) || '0', 10)
  const tabId = lastTabId + 1

  localStorage.setItem(propName, tabId.toString())
  sessionStorage.setItem(propName, tabId.toString())

  return tabId
}
