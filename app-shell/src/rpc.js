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

        pendingRequests[callId] = resolve

        channel.postMessage(JSON.stringify({
          callId,
          action,
          params,
        }))
      })
    },

    _callResult(callId, result) {
      const responseHandler = pendingRequests[callId]

      if (!responseHandler) {
        // eslint-disable-next-line no-console
        console.error(`RPC: got response for unknown call id ${callId}, ignoring`, result)
        return
      }

      responseHandler(result)

      delete pendingRequests[callId]
    },
  }

  if (window.onRPCReady) {
    window.onRPCReady()
  }
})('app-shell')
