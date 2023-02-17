import Comment from "./comment"
import { createElement } from "./utils"

export default class UnreviewedComments {
  element: HTMLDialogElement
  comments: CommentRecord[]
  loading = false

  constructor(element: HTMLDialogElement, comments: CommentRecord[]) {
    this.element = element
    this.comments = comments
    this.initUi()
  }

  open() { this.element.showModal() }

  initUi() {
    this.renderComments()
  }

  groupedComments(): {[key: string]: Comment[]} {
    return this.comments.reduce((groups: {[key: string]: Comment[]}, comment: CommentRecord) => {
      const group: Comment[] = groups[comment.page_path || '#'] || []
      const commentComponent = new Comment(comment)
      commentComponent.onApprove = (instance) => instance.destroy()
      commentComponent.onDelete = (instance) => {
        if (instance.element.parentElement?.childElementCount == 1) {
          instance.element.parentElement?.previousSibling?.remove()
          instance.element.parentElement?.remove()
        }
      }

      group.push(commentComponent)
      groups[comment.page_path || '#'] = group
      return groups
    }, {})
  }

  renderComments() {
    for (const [href, comments] of Object.entries(this.groupedComments())) {
      const list = createElement('ol', 'comments')
      const title = createElement('h4', 'unreviewed-page')
      const subtitle = createElement('small')
      subtitle.textContent = href
      const link = createElement('a', 'unreviewed-link', { href: `${href}#besedka` })
      link.textContent = comments[0].comment.page_title || 'Untitled'
      title.append(link, subtitle)
      comments.forEach(c => list.append(c.element))
      this.element.append(title, list)
    }
  }
}
