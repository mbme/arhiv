import {
  useRouter,
  paramAsArray,
  paramAsString,
} from '~/web-router'

export function useWorkspaceURLManager() {
  const router = useRouter()

  const openIds = paramAsArray(router.location$.value.params.id)
  const filter = paramAsString(router.location$.value.params.filter)

  return {
    filter,
    updateFilter(newFilter: string | undefined) {
      router.replaceParam('filter', newFilter || undefined)
    },

    openIds,
    openId(id: string) {
      if (openIds.includes(id)) {
        return
      }

      router.replaceParam('id', [...openIds, id])
    },
    closeId(id: string) {
      if (!openIds.includes(id)) {
        return
      }

      router.replaceParam('id', openIds.filter(openId => openId !== id))
    },
  }
}
