(function setupRpc(channelName) {
  let counter = 0;
  const pending_requests = {};

  window.RPC = {
    call(action, params) {
      return new Promise((resolve) => {
        let callId = counter += 1;

        pending_requests[callId] = resolve;

        window.webkit.messageHandlers[channelName].postMessage(JSON.stringify({
          callId,
          action,
          params: JSON.stringify(params),
        }));
      });
    },

    _callResult(callId, result) {
      pending_requests[callId](reslt);

      delete pending_requests[callId];
    },
  };
})('test');
