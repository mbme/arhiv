interface Window {
  __SERVER__: boolean
}

declare module NodeJS {
  interface Global {
    __SERVER__: boolean
  }
}
