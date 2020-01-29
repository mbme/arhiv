export interface IQueryParam {
  name: string
  value: string | undefined
}

export interface ILocation {
  path: string
  params: IQueryParam[]
}

export type SimpleLocation = { path: string, params?: IQueryParam[] } | string
