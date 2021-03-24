import { isObject } from '@v/utils'
import { IDataDescription, IDataSchema } from '../api'

export class DataManager {
  constructor(
    private _schema: IDataSchema,
  ) {}

  getModules(): readonly IDataDescription[] {
    return this._schema.modules
  }

  getDataDescription(documentType: string): IDataDescription {
    const dataDescription = this._schema.modules.find(item => item.documentType === documentType)

    if (!dataDescription) {
      throw new Error(`Unknown document type ${documentType}`)
    }

    return dataDescription
  }

  getMandatoryFields(documentType: string): string[] {
    const dataDescription = this.getDataDescription(documentType)

    return dataDescription.fields
      .filter(({ fieldType }) => isObject(fieldType) && 'Ref' in fieldType)
      .map(({ name }) => name)
  }
}
