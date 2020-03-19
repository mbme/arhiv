(function setupRpc(channelName) {
  let counter = 0
  const pendingRequests = {}

  window.RPC = {
    call(action, params = {}) {
      return new Promise((resolve) => {
        const callId = counter += 1

        pendingRequests[callId] = resolve

        window.webkit.messageHandlers[channelName].postMessage(JSON.stringify({
          callId,
          action,
          params,
        }))
      })
    },

    _callResult(callId, result) {
      pendingRequests[callId](result)

      delete pendingRequests[callId]
    },
  }
})('app-shell')
