import App from './app'
import {Comment} from './types'

export default class CommentContainer {
  element: HTMLElement
  app: App

  constructor(element: HTMLElement, app: App) {
    this.element = element
    this.app = app
  }

  add({ created_at, name, body, reviewed }: Comment, prepend = false) {
    const li = document.createElement('li')
    li.classList.add('besedka-comment')

    if (!reviewed) li.classList.add('besedka-unreviewed-comment')

    const author = document.createElement('span')
    const date = document.createElement('time')
    const text = document.createElement('span')
    const reply = document.createElement('button')

    author.textContent = name
    date.textContent = created_at.toLocaleString(navigator.language, { dateStyle: "medium", timeStyle: "short" })
    text.textContent = body
    reply.textContent = "Reply"

    li.append(author, date, text, reply)

    prepend ? this.element.prepend(li) : this.element.append(li)
  }
}
