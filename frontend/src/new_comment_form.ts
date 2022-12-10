import { createButton, createElement, getToken, message, request } from "./utils"

export default class NewCommentForm {
  element: HTMLElement
  button = createButton('Post', 'new-comment-post')
  body = createElement<HTMLTextAreaElement>('textarea', 'new-comment-body', { placeholder: 'Leave a comment' })
  message = createElement<HTMLDivElement>('div')
  name?: HTMLInputElement
  parentId?: number
  callback: Function

  constructor(element: HTMLFormElement, callback: (response: PostCommentResponse) => void, parentId?: number) {
    this.parentId = parentId
    this.element = element
    this.callback = callback

    this.initUi()
    this.attachEvents()
  }

  initUi() {
    if (!window.__besedka.user.name) {
      this.name = createElement<HTMLInputElement>('input', 'new-comment-author', { placeholder: 'Your name' })
      this.element.prepend(this.name)
    }

    const val = window.localStorage.getItem(this.storageKey())
    if (val) this.body.value = val
    this.button.innerText = "Post"

    this.element.append(this.body, this.button, this.message)
  }

  attachEvents() {
    this.element.addEventListener('submit', (e) => this.comment(e))
    this.body.addEventListener('change', () => this.save())
    this.body.addEventListener('keyup', () => this.save())
  }

  storageKey(): string {
    if (this.parentId) return `__besedka_reply_draft_${this.parentId}`

    return `__besedka_comment_draft_${window.__besedka.req.path}`
  }

  save() {
    window.localStorage.setItem(this.storageKey(), this.body.value)
  }

  reset() {
    this.body.value = ''
    window.localStorage.removeItem(this.storageKey())
  }

  url(): string {
    return this.parentId ? `/api/comment/${this.parentId}` : '/api/comment'
  }

  async comment(e: SubmitEvent) {
    e.preventDefault()
    if (!this.body.value.trim()) {
      message('What would you like to say?', 'error', this.message)
    } else {
      this.button.disabled = true

      const body = this.body.value
      const name = this.name?.value
      const token = getToken()

      try {
        const { json } = await request<PostCommentResponse>(this.url(), Object.assign({
          payload: { body, name, token }
        }, window.__besedka.req), 'POST', this.message)

        if (json) {
          this.callback(json)
          this.reset()
        }
      } finally {
        this.button.disabled = false
      }
    }
  }

  destroy() {
    this.reset()
    this.element.remove()
  }
}