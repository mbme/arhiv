import { useRouter } from '~/web-router'

function parseIds(ids: string): string[] {
  if (!ids.length) {
    return []
  }

  return ids.split('-')
}

export function useWorkspaceManager() {
  const router = useRouter()

  const {
    filter = '',
    ids: idsString = '',
  } = router.location$.value.params

  const openIds = parseIds(idsString)

  return {
    filter,
    updateFilter(newFilter: string | undefined) {
      router.replaceParam('filter', newFilter)
    },

    openIds,
    openId(id: string) {
      if (openIds.includes(id)) {
        return
      }

      router.replaceParam('ids', [...openIds, id].join('-'))
    },
    closeId(id: string) {
      if (!openIds.includes(id)) {
        return
      }

      router.replaceParam('ids', openIds.filter(openId => openId !== id).join('-'))
    },
  }
}
