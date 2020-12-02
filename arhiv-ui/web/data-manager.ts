import * as React from 'react'
import { Dict } from '@v/utils'
import { createContext } from '@v/web-utils'
import { IDataDescription } from './api'

export class DataManager {
  constructor(
    private _modules: Dict<IDataDescription>,
  ) {}

  getDataDescription(documentType: string): IDataDescription {
    const dataDescription = this._modules[documentType]

    if (!dataDescription) {
      throw new Error(`Unknown document type ${documentType}`)
    }

    return dataDescription
  }

  pickTitleField(documentType: string): string {
    const dataDescription = this.getDataDescription(documentType)

    const titleField = Object.entries(dataDescription.fields).find(([, field]) => field.fieldType === 'String')
    if (!titleField) {
      throw new Error("can't pick field for title")
    }

    return titleField[0]
  }
}

export const DataManagerContext = createContext<DataManager>()

export function useDataDescription(documentType: string) {
  const dataManager = DataManagerContext.use()

  return React.useMemo(() => ({
    dataDescription: dataManager.getDataDescription(documentType),
    titleField: dataManager.pickTitleField(documentType),
  }), [documentType])
}
