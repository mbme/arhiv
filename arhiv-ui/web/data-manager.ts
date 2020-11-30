import { Dict } from '@v/utils'
import { IDataDescription, IDocument } from './api'

export class DocumentDataManager {
  constructor(
    private _modules: Dict<IDataDescription>,
  ) {}

  private _getDataDescription(type: string): IDataDescription {
    const dataDescription = this._modules[type]

    if (!dataDescription) {
      throw new Error(`Unknown module type ${type}`)
    }

    return dataDescription
  }

  pickTitle(document: IDocument): string {
    const dataDescription = this._getDataDescription(document.data.type)

    const titleField = Object.entries(dataDescription.fields).find(([, field]) => field.fieldType === 'String')
    if (!titleField) {
      throw new Error("can't pick field for title")
    }

    return document.data[titleField[0]] as string
  }

}
