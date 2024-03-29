import { createButton, createElement, message, request, safeParse } from "./utils"
import UnreviewedComments from "./unreviewed_comments"

export default class ModeratorControls {
  element: HTMLDivElement

  constructor(element: HTMLDivElement) {
    this.element = element
    this.initUi()
  }

  initUi() {
    this.element.innerHTML = ''
    if (this.moderator()) {
      this.loadUnreviewedComments()
      this.buildLock()
    }

    if (this.loggedModerator()) this.buildLogout()
    if (!this.loggedModerator() && !this.signedUser()) this.buildLogin()
  }

  moderator(): boolean {
    return window.__besedka.user.moderator === true
  }

  signedUser(): boolean {
    return !!window.__besedka.user.name
  }

  loggedModerator(): boolean {
    return !!window.__besedka.req.sid
  }

  buildLogout() {
    const logout = createButton('Logout', 'logout', { title: 'Logout' })
    this.element.append(logout)
    logout.addEventListener('click', () => {
      window.localStorage.removeItem('__besedka_mod')
      window.__besedka.user = safeParse(document.getElementById('besedka-user')?.innerText)
      window.__besedka.req.sid = undefined
      window.__besedka.run()
    })
  }

  buildModal(): { msg: HTMLDivElement, modal: HTMLDialogElement} {
    const msg = createElement<HTMLDivElement>('div', 'message')
    const modal = createElement<HTMLDialogElement>('dialog', 'modal')
    const close = createButton('×', 'close-modal')
    close.addEventListener('click', () => modal.close())
    modal.append(close, msg)
    return { msg, modal }
  }

  buildLogin() {
    const loginButton = createButton('Login', 'login', { title: 'Login' })
    const { msg, modal } = this.buildModal()
    const form = createElement<HTMLFormElement>('form', 'login-form')
    const name = createElement<HTMLInputElement>('input', 'login-name', { placeholder: 'User' })
    const pass = createElement<HTMLInputElement>('input', 'login-password', { placeholder: 'Pass', type: 'password' })
    const login = createButton('Login', '', { title: 'Login' })

    form.append(name, pass, login)
    modal.append(form)

    loginButton.addEventListener('click', () => {
      modal.showModal()
      name.focus()
    })
    form.addEventListener('submit', async (e) => {
      e.preventDefault()
      if (name.value && pass.value) {
        try {
          login.disabled = true
          const { status, json } = await request<LoginResponse>(
            '/api/login',
            { name: name.value, password: pass.value },
            'POST',
            msg
          )
          if (status === 401) {
            message("Invalid credentials", "error", msg)
          } else if(json) {
            const user = { name: json.name, sid: json.sid, avatar: json.avatar, moderator: true, op: json.op }
            window.localStorage.setItem('__besedka_mod', JSON.stringify(user))
            window.__besedka.user = user
            window.__besedka.req.sid = json.sid
            window.__besedka.run()
          }
        } finally {
          login.disabled = false
        }
      } else {
        message("Please provide a login name and a password", "error", msg)
      }
    })

    this.element.append(loginButton, modal)
  }

  buildLock() {
    const locked = window.__besedka.config?.locked
    const [text, klass] = locked ? ['Unlock page', 'unlock-page'] : ['Lock page', 'lock-page']
    const lock = createButton(text, klass, { title: text })
    lock.addEventListener('click', async () => {
      lock.disabled = true
      try {
        const { status } = await request<Config>('/api/pages', window.__besedka.req, 'PATCH')
        if (status == 200) window.__besedka.run()
      } finally {
        lock.disabled = false
      }
    })

    this.element.append(lock)
  }

  buildUnreviewed(comments: CommentRecord[]) {
    const button = createButton('View unreviewed comments', 'view-unreviewed-comments')
    const { modal } = this.buildModal()
    const unreviewed = new UnreviewedComments(modal, comments)
    this.element.append(modal)
    this.element.append(button)
    button.addEventListener('click', () => unreviewed.open())
  }

  async loadUnreviewedComments() {
    const dummyMessage = createElement<HTMLDivElement>('div')
    const { status, json } = await request<CommentRecord[]>('/api/comments/unreviewed', window.__besedka.req, 'POST', dummyMessage)

    if (status != 404 && json && json?.length != 0) this.buildUnreviewed(json)
  }
}
