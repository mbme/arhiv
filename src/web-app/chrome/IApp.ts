import { IParams } from '~/web-router'

export interface IApp {
  name: string,
  route: string,
  render(params: IParams): React.ReactNode,
}
