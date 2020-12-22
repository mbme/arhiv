import { merge } from '@v/utils'

export interface IUIOptions {
  catalog: {
    pageSize?: number
  }

  catalogEntry: {
    showModificationDate: boolean
    showDataFields: string[]
  }
}

const DEFAULT_UI_OPTIONS: IUIOptions = {
  catalog: {
    pageSize: 12,
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
    pageSize: undefined
  },

  catalogEntry: {
    showModificationDate: false,
    showDataFields: ['complexity', 'status'],
  }
}

export function getUIOptions(documentType: string): IUIOptions {
  return merge<IUIOptions>(DEFAULT_UI_OPTIONS, (UI_OVERRIDES[documentType] || {}) as any)
}
