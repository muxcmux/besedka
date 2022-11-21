import {Comment} from './types'

export default class CommentContainer {
  element: HTMLElement

  constructor(element: HTMLElement) {
    this.element = element
  }

  add({ body }: Comment) {
    const div = document.createElement('div')
    div.textContent = body
    this.element.appendChild(div)
  }
}
