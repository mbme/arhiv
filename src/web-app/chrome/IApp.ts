import { IParams } from '~/web-router'

export interface IApp {
  name: string,
  rootRoute: string,
  routes: {
    [route: string]: (params: IParams) => React.ReactNode,
  },
}
