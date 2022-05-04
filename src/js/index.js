function _ipc_handler(_event, params) {
  console.info("evoking ipc: ", _event);
  let _params = params || {};
  window.ipc.postMessage(JSON.stringify(Object.assign({ _event }, _params)));
}

document.addEventListener("click", (e) => {
  if (e.target.matches("a")) {
    e.preventDefault();
    if (e.target.getAttribute("target") === "_blank") {
      _ipc_handler("open-browser", e.target.getAttribute("href"));
    }
  }
});

window.onload = function () {
  _ipc_handler("set-conversion-factor", {
    conversion_factor:
      parseFloat(getComputedStyle(document.documentElement).fontSize) / 16,
  });
};
