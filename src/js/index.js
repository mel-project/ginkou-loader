function _ipc_handler(_event, params) {
    window.ipc.postMessage(JSON.stringify(Object.assign(params, {_event})))
}
window.onload = function() {
   _ipc_handler('set_conversion_factor', {conversion_factor: parseFloat(getComputedStyle(document.documentElement).fontSize) / 16});
} 