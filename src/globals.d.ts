/* tslint:disable */
interface Window {
  __SERVER__: boolean
}

declare var _h: any // inferno-hyperscript

declare module NodeJS {
  interface Global {
    __SERVER__: boolean
  }
}
