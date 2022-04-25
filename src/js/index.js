function _ipc_handler(_event, params) {
    console.info('evoking ipc: ', _event)
    let _params = params || {}
    window.ipc.postMessage(JSON.stringify(Object.assign({_event},_params)))
}
window.onload = function() {
   _ipc_handler('set-conversion-factor', {conversion_factor: parseFloat(getComputedStyle(document.documentElement).fontSize) / 16});
} 