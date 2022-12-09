import App from './app'
import { safeParse } from './utils'

document.addEventListener('DOMContentLoaded', () => {
  const container = document.getElementById('besedka') as HTMLDivElement

  if (container) {
    const configContainer = document.getElementById('besedka-config')
    const userContainer = document.getElementById('besedka-user')

    let config = configContainer?.innerText
    let user = userContainer?.innerText

    let { site, path } = safeParse(config)

    const mod = window.localStorage.getItem('__besedka_mod')
    let logged: LoginResponse | undefined
    if (mod) { logged = JSON.parse(mod) }

    let userObject = safeParse(user)

    if (logged) {
      userObject = { name: logged.name, avatar: logged.avatar, moderator: true }
    }

    if (user) user = btoa(user.trim())

    let signature = userContainer?.dataset?.signature

    const req: ApiRequest = {
      site: site || window.location.hostname,
      path: path || window.location.pathname,
      user,
      signature,
      sid: logged?.sid
    }

    window.__besedka = new App(container, req, userObject)
    window.__besedka.run()
  }
})
