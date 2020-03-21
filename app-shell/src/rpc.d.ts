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
  // eslint-disable-next-line @typescript-eslint/interface-name-prefix
  interface Window {
    RPC: IRPC
    onRPCReady?: () => void
  }
}

export {}
