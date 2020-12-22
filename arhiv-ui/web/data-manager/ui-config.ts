export interface IUIOptions {
  catalog?: {
    pageSize?: number
  }

  catalogEntry?: {
    showModificationDate?: boolean
  }
}

export const DEFAULT_UI_OPTIONS: IUIOptions = {
  catalog: {
    pageSize: 12,
  },

  catalogEntry: {
    showModificationDate: true,
  },
}

export const UIConfig: Record<string, IUIOptions> = {}

UIConfig['project'] = {
  catalog: {
    pageSize: undefined
  },

  catalogEntry: {
    showModificationDate: false
  }
}

UIConfig['task'] = {
  catalog: {
    pageSize: undefined
  },

  catalogEntry: {
    showModificationDate: false
  }
}
