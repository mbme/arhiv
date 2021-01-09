import { isObject } from '@v/utils'
import { IDataDescription, IDataSchema } from '../api'

export class DataManager {
  constructor(
    private _schema: IDataSchema,
  ) {}

  getDataDescription(documentType: string): IDataDescription {
    const dataDescription = this._schema.modules.find(item => item.documentType === documentType)

    if (!dataDescription) {
      throw new Error(`Unknown document type ${documentType}`)
    }

    return dataDescription
  }

  pickTitleField(documentType: string): string {
    const dataDescription = this.getDataDescription(documentType)

    const titleField = dataDescription.fields.find(({ fieldType }) => 'String' in fieldType)
    if (!titleField) {
      throw new Error("can't pick field for title")
    }

    return titleField.name
  }

  getMandatoryFields(documentType: string): string[] {
    const dataDescription = this.getDataDescription(documentType)

    return dataDescription.fields
      .filter(({ fieldType }) => isObject(fieldType) && 'Ref' in fieldType)
      .map(({ name }) => name)
  }
}
