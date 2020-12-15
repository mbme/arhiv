(function setupRpc(channelName) {
  let counter = 0
  const pendingRequests = {}

  const channel = window.webkit && window.webkit.messageHandlers[channelName]
  if (!channel) {
    throw new Error(`setupRpc failed: message handler ${channelName} is missing`)
  }

  window.RPC = {
    call(action, params = {}) {
      return new Promise((resolve) => {
        const callId = counter += 1

        pendingRequests[callId] = { resolve, reject }

        channel.postMessage(JSON.stringify({
          callId,
          action,
          params,
        }))
      })
    },

    _callResponse(response) {
      const {
        callId,
        result,
        err,
      } = response

      const responseHandlers = pendingRequests[callId]

      if (!responseHandlers) {
        // eslint-disable-next-line no-console
        console.error(`RPC: got response for unknown call id ${callId}, ignoring`, result)
        return
      }

      if (err) {
        responseHandlers.reject(err)
      } else {
        responseHandlers.resolve(result)
      }

      delete pendingRequests[callId]
    },
  }

  window.RPC_PROXY = new Proxy({}, {
    get(_, prop) {
      return params => window.RPC.call(prop, params)
    }
  })
})('app-shell')
