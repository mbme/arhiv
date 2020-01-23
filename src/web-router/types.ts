export type QueryParamType = string | readonly string[] | undefined

export interface IParams {
  [key: string]: QueryParamType
}

export interface ILocation {
  path: string
  params: IParams
}

export type SimpleLocation = { path: string, params?: IParams } | string
