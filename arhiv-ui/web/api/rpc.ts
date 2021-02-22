interface IRpcMessageResponse {
  result: unknown
  err?: string
}

const RPC = {
  async call(action: string, params = {}) {
    const response: IRpcMessageResponse = await fetch('/rpc', {
      method: 'POST',
      cache: 'no-cache',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        action,
        params,
      })
    }).then(response => response.json())

    if (response.err) {
      return Promise.reject(response.err)
    }

    return response.result
  },
}

export const RPC_PROXY = new Proxy({}, {
  get(_, prop: string) {
    return (params: Record<string, any>) => RPC.call(prop, params)
  }
})
