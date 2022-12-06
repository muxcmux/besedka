import App from "./app"
import { PostCommentResponse } from "./types"
import { getToken, message, post, setToken } from "./utils"

export default class NewComment {
  app: App
  container: HTMLElement
  // @ts-ignore strictPropertyInitialization
  button: HTMLButtonElement
  // @ts-ignore strictPropertyInitialization
  body: HTMLTextAreaElement
  // @ts-ignore strictPropertyInitialization
  name: HTMLInputElement | undefined

  constructor(container: HTMLElement, app: App) {
    this.app = app
    this.container = container
    this.initUi()
    this.attachEvents()
  }

  initUi() {
    if (!this.app.user.name) {
      this.name = document.createElement('input')
      this.name.setAttribute('placeholder', 'Your name')
      this.container.prepend(this.name)
    }

    this.body = document.createElement('textarea')
    const val = window.localStorage.getItem('__besedka_comment_backup')
    if (val) this.body.value = val
    this.body.setAttribute('placeholder', 'Leave a comment')
    this.container.append(this.body)

    this.button = document.createElement('button')
    this.button.innerText = "Post"
    this.container.append(this.button)
  }

  attachEvents() {
    this.button.addEventListener('click', e => this.comment(e))
    this.body.addEventListener('change', e => this.save(e))
    this.body.addEventListener('keyup', e => this.save(e))
  }

  save(_e: Event) {
    window.localStorage.setItem('__besedka_comment_backup', this.body.value)
  }

  reset() {
    this.body.value = ''
    window.localStorage.removeItem('__besedka_comment_backup')
  }

  async comment(_e: MouseEvent) {
    if (!this.body.value.trim()) {
      message('What would you like to say?')
    } else {
      this.button.disabled = true

      const body = this.body.value
      const name = this.name?.value
      const token = getToken()

      try {
        const { json } = await post<PostCommentResponse>('/api/comment', Object.assign({
          payload: { body, name, token }
        }, this.app.req))

        if (json) {
          setToken(json.token)
          this.app.setConfig(json.site)
          this.app.comments.add(json.comment, true)
          this.reset()
        }
      } finally {
        this.button.disabled = false
      }
    }
  }
}
