import { Dict } from '@v/utils'
import { createContext } from '@v/web-utils'
import { IDataDescription, IDocument } from './api'

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

  pickTitle(document: IDocument): string {
    const dataDescription = this.getDataDescription(document.data.type)

    const titleField = Object.entries(dataDescription.fields).find(([, field]) => field.fieldType === 'String')
    if (!titleField) {
      throw new Error("can't pick field for title")
    }

    return document.data[titleField[0]] as string
  }

}

export const DataManagerContext = createContext<DataManager>()

export function useDataDescription(documentType: string): IDataDescription {
  const dataManager = DataManagerContext.use()

  return dataManager.getDataDescription(documentType)
}
