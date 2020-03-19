type Json =
  | null
  | boolean
  | number
  | string
  | Json[]
  | { [prop: string]: Json }

interface IRPC {
  call(action: string, params?: Json): Promise<Json>
}

declare global {
  interface Window {
    RPC: IRPC
  }
}

export {}
