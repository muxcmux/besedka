import App from './app'
import {ApiRequest, User} from './types'
import { safeParse } from './utils'


document.addEventListener('DOMContentLoaded', () => {
  const container = document.getElementById('besedka')

  if (container) {
    const configContainer = document.getElementById('besedka-config')
    const userContainer = document.getElementById('besedka-user')

    let config = configContainer?.innerText
    let user = userContainer?.innerText

    let { site, path } = safeParse(user)
    let userObject: User = safeParse(user)

    if (config) config = btoa(config)
    if (user) user = btoa(user)

    let signature = configContainer?.dataset?.signature

    const req: ApiRequest = {
      site: site || window.location.hostname,
      path: path || window.location.pathname,
      user,
      signature
    }

    new App(container, req, userObject)
  }
})
