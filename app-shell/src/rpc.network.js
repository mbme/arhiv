(function setupRpc() {
  let counter = 0

  window.RPC = {
    async call(action, params = {}) {
      const callId = counter += 1

      const response = await fetch('/rpc', {
        method: 'POST',
        cache: 'no-cache',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          callId,
          action,
          params,
        })
      }).then(response => response.json())

      return response.result
    },
  }

  window.RPC_PROXY = new Proxy({}, {
    get(_, prop) {
      return params => window.RPC.call(prop, params)
    }
  })
})()
