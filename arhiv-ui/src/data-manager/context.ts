import * as React from 'react'
import { createContext } from '@v/web-utils'
import { DataManager } from './data-manager'

export const DataManagerContext = createContext<DataManager>()

export function useDataDescription(documentType: string) {
  const dataManager = DataManagerContext.use()

  return React.useMemo(() => ({
    dataDescription: dataManager.getDataDescription(documentType),
    mandatoryFields: dataManager.getMandatoryFields(documentType),
  }), [documentType])
}
