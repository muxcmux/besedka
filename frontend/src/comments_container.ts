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

    author.textContent = name
    date.textContent = created_at.toLocaleString()
    text.textContent = body

    li.append(author, date, text)

    prepend ? this.element.prepend(li) : this.element.append(li)
  }
}
