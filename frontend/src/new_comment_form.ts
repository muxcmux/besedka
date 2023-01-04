import { createButton, createElement, debounce, getToken, request } from "./utils"

export default class NewCommentForm<R> {
  element: HTMLElement
  loading = false
  previewing = false
  button = createButton('Review comment', 'post-comment-button')
  editButton = createButton('Make edits', 'make-edits-button', { type: 'button' })
  body = createElement<HTMLTextAreaElement>('textarea', 'comment-textarea', { placeholder: 'Leave a comment' })
  previewBody = createElement<HTMLDivElement>('div', 'comment-preview')
  previewAuthor = createElement<HTMLDivElement>('div', 'author-preview')
  previewTimestamp = createElement<HTMLTimeElement>('time', 'timestamp-preview')
  avatar = createElement<HTMLDivElement>('div', 'avatar no-avatar')
  name?: HTMLInputElement
  parentId?: number
  callback: Function

  constructor(
    element: HTMLFormElement,
    callback: (response: R) => void,
    parentId?: number
  ) {
    this.parentId = parentId
    this.element = element
    this.callback = callback

    this.init()
    this.initUi()
    this.attachEvents()
  }

  init() {
    const val = window.localStorage.getItem(this.storageKey())
    if (val) this.body.value = val
  }

  initUi() {
    this.element.append(this.avatar)
    if (window.__besedka.user.avatar) {
      this.avatar.classList.remove('besedka-no-avatar')
      this.avatar.append(createElement<HTMLImageElement>('img', '', { src: window.__besedka.user.avatar, loading: 'lazy' }))
    }

    if (!window.__besedka.user.name) {
      this.name = createElement<HTMLInputElement>('input', 'comment-author-input', { placeholder: 'Anonymous' })
      this.element.classList.add('besedka-anonymous-user')
      this.element.append(this.name)
    }

    this.element.append(
      this.body, this.previewBody, this.previewAuthor,
      this.previewTimestamp, this.button, this.editButton
    )
  }

  attachEvents() {
    this.element.addEventListener('submit', (e) => this.submit(e))
    this.editButton.addEventListener('click', () => {
      this.element.classList.remove('besedka-previewing')
      this.previewing = false
      this.button.textContent = 'Review comment'
      this.body.focus()
    })
    this.body.addEventListener('change', debounce(() => this.save()))
    this.body.addEventListener('keyup', debounce(() => this.save()))
    this.body.addEventListener('focus', () => this.element.classList.add('besedka-focused'))
    this.name?.addEventListener('focus', () => this.element.classList.add('besedka-focused'))
    this.body.addEventListener('blur', () => this.element.classList.remove('besedka-focused'))
    this.name?.addEventListener('blur', () => this.element.classList.remove('besedka-focused'))
  }

  storageKey(): string {
    if (this.parentId) return `__besedka_reply_draft_${this.parentId}`

    return `__besedka_comment_draft_${window.__besedka.req.path}`
  }

  save() {
    if (this.body.value.trim()) this.element.classList.remove('besedka-comment-error')
    window.localStorage.setItem(this.storageKey(), this.body.value)
  }

  reset() {
    this.body.value = ''
    window.localStorage.removeItem(this.storageKey())
  }

  url(): string {
    return this.parentId ? `/api/comment/${this.parentId}` : '/api/comment'
  }

  method(): string { return 'POST' }

  async submit(e: SubmitEvent) {
    if (!this.loading) {
      e.preventDefault()
      this.element.classList.remove('besedka-comment-error')

      if (!this.body.value.trim()) {
        this.element.classList.add('besedka-comment-error')
        this.body.focus()
      } else {
        this.loading = true
        this.element.classList.add('besedka-loading')
        this.button.disabled = true
        try {
          await (this.previewing ? this.comment() : this.showPreview())
        } finally {
          this.loading = false
          this.element.classList.remove('besedka-loading')
          this.button.disabled = false
        }
      }
    }
  }

  async comment() {
    const body = this.body.value
    const name = this.name?.value
    const token = getToken()

    try {
      const { json } = await request<R>(this.url(), Object.assign({
        payload: { body, name, token }
      }, window.__besedka.req), this.method())
      if (json) {
        this.callback(json)
        this.reset()
      }
    } catch {
      this.element.classList.add('besedka-comment-error')
      this.body.focus()
    } finally {
      this.element.classList.remove('besedka-previewing')
      this.button.textContent = 'Review comment'
      this.previewing = false
    }
  }

  async showPreview() {
    try {
      const { text } = await request<R>('/api/preview', Object.assign({
        payload: this.body.value
      }, window.__besedka.req), 'POST')

      this.previewBody.innerHTML = text
      this.previewAuthor.textContent = this.name ? this.name.value.trim() || 'Anonymous' : window.__besedka.user.name!
      this.previewTimestamp.textContent = "just now"
      this.element.classList.add('besedka-previewing')
      this.previewing = true
      this.button.textContent = "Send"
    } catch {
      this.element.classList.add('besedka-comment-error')
      this.element.classList.remove('besedka-previewing')
      this.previewing = true
      this.body.focus()
    }
  }

  destroy() {
    this.reset()
    this.element.remove()
  }
}
