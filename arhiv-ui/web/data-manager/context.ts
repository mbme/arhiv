import * as React from 'react'
import { merge } from '@v/utils'
import { createContext } from '@v/web-utils'
import { DEFAULT_UI_OPTIONS, UIConfig } from './ui-config'
import { DataManager } from './data-manager'

export const DataManagerContext = createContext<DataManager>()

export function useDataDescription(documentType: string) {
  const dataManager = DataManagerContext.use()

  // FIXME split into separate hooks
  return React.useMemo(() => ({
    dataDescription: dataManager.getDataDescription(documentType),
    titleField: dataManager.pickTitleField(documentType),
    mandatoryFields: dataManager.getMandatoryFields(documentType),
    uiOptions: merge(DEFAULT_UI_OPTIONS, UIConfig[documentType] || {}),
  }), [documentType])
}
