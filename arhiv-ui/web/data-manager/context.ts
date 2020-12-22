import * as React from 'react'
import { createContext } from '@v/web-utils'
import { getUIOptions } from './ui-config'
import { DataManager } from './data-manager'

export const DataManagerContext = createContext<DataManager>()

export function useDataDescription(documentType: string) {
  const dataManager = DataManagerContext.use()

  // FIXME split into separate hooks
  return React.useMemo(() => ({
    dataDescription: dataManager.getDataDescription(documentType),
    titleField: dataManager.pickTitleField(documentType),
    mandatoryFields: dataManager.getMandatoryFields(documentType),
    uiOptions: getUIOptions(documentType),
  }), [documentType])
}
