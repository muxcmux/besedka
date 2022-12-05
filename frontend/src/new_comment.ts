import App from "./app"
import { PostCommentResponse } from "./types"
import { post, reviver } from "./utils"

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
    this.body.setAttribute('placeholder', 'Leave a comment')
    this.container.append(this.body)

    this.button = document.createElement('button')
    this.button.innerText = "Post"
    this.container.append(this.button)
  }

  attachEvents() {
    this.button.addEventListener('click', e => this.comment(e))
  }

  reset() {
    this.body.value = ''
  }

  async comment(_e: Event) {
    if (!this.body.value) {
      this.app.error('What would you like to say?')
    } else {
      this.button.disabled = true
      const body = this.body.value
      const name = this.name?.value
      const token = this.app.getToken()

      try {
        const request = await post('/api/comment', Object.assign({
          payload: { body, name, token }
        }, this.app.req))

        if (request.ok) {
          const response = JSON.parse(await request.text(), reviver) as PostCommentResponse
          this.app.setToken(response.token)
          this.app.comments.add(response.comment, true)
        }
      } catch {
        this.app.error("Unable to post a comment at this time")
      } finally {
        this.button.disabled = false
      }
    }
  }
}
