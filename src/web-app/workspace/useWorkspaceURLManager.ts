import {
  RouterContext,
  paramAsArray,
  paramAsString,
  updateParam,
} from '~/web-router'

export function useWorkspaceURLManager() {
  const router = RouterContext.use()

  const { params } = router.location$.value

  const openIds = paramAsArray(params, 'id')
  const filter = paramAsString(params, 'filter')

  return {
    params,

    filter,
    updateFilter(newFilter: string | undefined) {
      router.replaceParams(updateParam(params, 'filter', newFilter || undefined))
    },

    openIds,
    openId(id: string) {
      if (openIds.includes(id)) {
        return
      }

      router.replaceParams([...params, { name: 'id', value: id }])
    },
    closeId(id: string) {
      if (!openIds.includes(id)) {
        return
      }

      router.replaceParams(params.filter(param => param.name !== 'id' || param.value !== id))
    },
  }
}
