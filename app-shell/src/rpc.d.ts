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
  interface Window {
    RPC: IRPC
    RPC_PROXY: unknown
    RPC_URL?: string
    JS_VARIABLES: Record<string, unknown>
  }
}

export {}
