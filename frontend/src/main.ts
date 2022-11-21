import App from './app'
import {ConfigRequest} from './types'

document.addEventListener('DOMContentLoaded', () => {
  const container = document.getElementById('besedka')

  if (container) {
    const configContainer = document.getElementById('besedka-config')
    let config = configContainer?.innerText
    if (config) config = btoa(config)
    let signature = configContainer?.dataset?.signature

    const cfg: ConfigRequest = {
      site: window.location.hostname,
      path: window.location.pathname,
      config,
      signature
    }

    new App(container, cfg)
  }
})
