import { merge } from '@v/utils'
import { OrderBy } from '../../api'

export interface ICatalogOptions {
  pageSize?: number
  order: OrderBy[]
  groupByField?: string
  showEntryModificationDate: boolean
  showEntryDataFields: string[]
}

const DEFAULT_UI_OPTIONS: ICatalogOptions = {
  pageSize: 12,
  order: [{ UpdatedAt: { asc: false } }],
  groupByField: undefined,
  showEntryModificationDate: true,
  showEntryDataFields: [],
}

const OVERRIDES: Record<string, Partial<ICatalogOptions>> = {}

OVERRIDES['project'] = {
  pageSize: undefined,
  showEntryModificationDate: false,
}

OVERRIDES['task'] = {
  pageSize: undefined,
  groupByField: 'status',
  order: [
    {
      EnumField: {
        selector: '$.status',
        asc: true,
        enumOrder: ['Inbox', 'InProgress', 'Paused', 'Todo', 'Done', 'Later', 'Cancelled' ],
      },
    },
    {
      UpdatedAt: {
        asc: false,
      },
    },
  ],
  showEntryModificationDate: false,
  showEntryDataFields: ['complexity', 'status'],
}

export function getUIOptions(documentType: string): ICatalogOptions {
  return merge<ICatalogOptions>(DEFAULT_UI_OPTIONS, (OVERRIDES[documentType] || {}) as any)
}
