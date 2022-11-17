import App from './app'
import {ConfigRequest} from './types'

document.addEventListener('DOMContentLoaded', () => {
  const container = document.getElementById('besedka')

  if (container) {
    const configContainer = document.getElementById('besedka-config')
    const config = configContainer?.innerText
    const signature = configContainer?.dataset?.signature

    const cfg: ConfigRequest = {
      site: window.location.hostname,
      path: window.location.pathname,
      config,
      signature
    }

    const app = new App(container, cfg)
    app.run()
  }
})
