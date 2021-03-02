import { merge } from '@v/utils'
import { OrderBy } from '../api'

export interface IUIOptions {
  catalog: {
    pageSize?: number
    order: OrderBy[]
    groupByField?: string
    skipAddDocumentAction?: boolean
    showEntryModificationDate: boolean
    showEntryDataFields: string[]
  }
}

const DEFAULT_UI_OPTIONS: IUIOptions = {
  catalog: {
    pageSize: 12,
    order: [{ UpdatedAt: { asc: false } }],
    groupByField: undefined,
    skipAddDocumentAction: undefined,
    showEntryModificationDate: true,
    showEntryDataFields: [],
  },
}

interface IUIOptionsOverrides {
  catalog?: Partial<IUIOptions['catalog']>
}

const UI_OVERRIDES: Record<string, IUIOptionsOverrides> = {}

UI_OVERRIDES['project'] = {
  catalog: {
    pageSize: undefined,
    showEntryModificationDate: false,
  },
}

UI_OVERRIDES['task'] = {
  catalog: {
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
  },
}

UI_OVERRIDES['attachment'] = {
  catalog: {
    skipAddDocumentAction: true,
  },
}

export function getUIOptions(documentType: string): IUIOptions {
  return merge<IUIOptions>(DEFAULT_UI_OPTIONS, (UI_OVERRIDES[documentType] || {}) as any)
}
