import { merge } from '@v/utils'
import { OrderBy } from '../api'

export interface IUIOptions {
  catalog: {
    pageSize?: number
    order: OrderBy[]
  }

  catalogEntry: {
    showModificationDate: boolean
    showDataFields: string[]
  }
}

const DEFAULT_UI_OPTIONS: IUIOptions = {
  catalog: {
    pageSize: 12,
    order: [{ UpdatedAt: { asc: false } }],
  },

  catalogEntry: {
    showModificationDate: true,
    showDataFields: [],
  },
}

interface IUIOptionsOverrides {
  catalog?: Partial<IUIOptions['catalog']>
  catalogEntry?: Partial<IUIOptions['catalogEntry']>
}

const UI_OVERRIDES: Record<string, IUIOptionsOverrides> = {}

UI_OVERRIDES['project'] = {
  catalog: {
    pageSize: undefined
  },

  catalogEntry: {
    showModificationDate: false
  }
}

UI_OVERRIDES['task'] = {
  catalog: {
    pageSize: undefined,
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
  },

  catalogEntry: {
    showModificationDate: false,
    showDataFields: ['complexity', 'status'],
  }
}

export function getUIOptions(documentType: string): IUIOptions {
  return merge<IUIOptions>(DEFAULT_UI_OPTIONS, (UI_OVERRIDES[documentType] || {}) as any)
}
