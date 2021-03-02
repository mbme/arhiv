import { merge } from '@v/utils'
import { OrderBy } from '../../api'

export interface ICatalogOptions {
  pageSize?: number
  order: OrderBy[]
  groupByField?: string
  showEntryModificationDate: boolean
  showEntryDataFields: string[]
}

const DEFAULT_UI_OPTIONS: ICatalogOptions = {
  pageSize: 12,
  order: [{ UpdatedAt: { asc: false } }],
  groupByField: undefined,
  showEntryModificationDate: true,
  showEntryDataFields: [],
}

export type CatalogOptionsOverrides = Partial<ICatalogOptions>

export function getUIOptions(catalogOptions: CatalogOptionsOverrides = {}): ICatalogOptions {
  return merge<ICatalogOptions>(DEFAULT_UI_OPTIONS, catalogOptions as any)
}
