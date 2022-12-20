import { createButton, createElement, debounce, getToken, request } from "./utils"

export default class NewCommentForm<R> {
  element: HTMLElement
  loading = false
  button = createButton('Post', 'post-comment-button')
  previewButton = createButton('Preview', 'preview-button', { type: 'button' })
  writeButton = createButton('Write', 'write-button', { type: 'button' })
  body = createElement<HTMLTextAreaElement>('textarea', 'comment-textarea', { placeholder: 'Leave a comment' })
  preview = createElement<HTMLDivElement>('div', 'comment-preview')
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

    this.element.append(this.body, this.preview, this.button, this.previewButton, this.writeButton)
  }

  attachEvents() {
    this.element.addEventListener('submit', (e) => this.comment(e))
    this.previewButton.addEventListener('click', () => this.showPreview())
    this.writeButton.addEventListener('click', () => {
      this.element.classList.remove('besedka-previewing')
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

  async comment(e: SubmitEvent) {
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
          this.element.classList.remove('besedka-previewing')
        } catch {
          this.element.classList.add('besedka-comment-error')
          this.body.focus()
        } finally {
          this.button.disabled = false
          this.loading = false
          this.element.classList.remove('besedka-loading')
        }
      }
    }
  }

  async showPreview() {
    if (!this.loading) {
      if (!this.body.value.trim()) {
        this.element.classList.add('besedka-comment-error')
        this.body.focus()
      } else {
        this.loading = true
        this.element.classList.add('besedka-loading')
        try {
          const { text } = await request<R>('/api/preview', Object.assign({
            payload: this.body.value
          }, window.__besedka.req), 'POST')

          this.element.classList.add('besedka-previewing')
          this.preview.innerHTML = text
        } catch {
          this.element.classList.add('besedka-comment-error')
          this.element.classList.remove('besedka-previewing')
        } finally {
          this.loading = false
          this.element.classList.remove('besedka-loading')
        }
      }
    }
  }

  destroy() {
    this.reset()
    this.element.remove()
  }
}
