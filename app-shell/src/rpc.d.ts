type Json =
  | null
  | boolean
  | number
  | string
  | Json[]
  | { [prop: string]: Json }

interface IRPC {
  call<T = Json>(action: string, params?: any): Promise<T>
}

declare global {
  // eslint-disable-next-line @typescript-eslint/interface-name-prefix
  interface Window {
    RPC: IRPC
  }
}

export {}
