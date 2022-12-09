import NewCommentForm from "./new_comment_form"
import { createButton, createElement } from "./utils"

export default class Comment {
  comment: CommentRecord
  replyForm?: NewCommentForm
  replyButton?: HTMLButtonElement

  replies = createElement('ol', 'replies')
  element = createElement('li', 'comment')
  author  = createElement('span', 'comment-author')
  date    = createElement('span', 'comment-timestamp')
  body    = createElement('span', 'comment-body')

  constructor(comment: CommentRecord) {
    this.comment = comment
    this.buildComment(this.comment)

    this.buildReplies()

    if (this.canReply()) this.element.append(this.createReplyButton())
  }

  buildReplies() {
    if (this.comment.replies?.length) {
      this.comment.replies?.forEach(reply => {
        const nested = new Comment(reply)
        this.replies!.append(nested.element)
      })
    }
  }

  canReply() {
    return !this.comment.parent_id && !this.comment.locked
  }

  buildComment({ created_at, body, name, reviewed, locked }: CommentRecord) {
    if (!reviewed) this.element.classList.add('besedka-unreviewed-comment')
    if (locked) this.element.classList.add('besedka-locked-comment')

    this.author.textContent = name
    this.date.textContent = created_at.toLocaleString(navigator.language, { dateStyle: "medium", timeStyle: "short" })
    this.body.textContent = body

    this.element.append(this.author, this.date, this.body, this.replies)
  }

  createReplyButton(): HTMLButtonElement {
    this.replyButton = createButton('Reply', 'add-reply')
    this.replyButton.addEventListener('click', e => this.openReplyForm(e))
    return this.replyButton
  }

  openReplyForm(e: MouseEvent) {
    (e.target as HTMLButtonElement).hidden = true

    const reply = createElement<HTMLFormElement>('form', 'new-reply')

    this.replyForm = new NewCommentForm(reply, ({ comment }) => {
      this.replies.append(new Comment(comment).element)
      this.closeReplyForm()
    }, this.comment.id)

    const cancel = createButton('Cancel', 'cancel-reply')
    cancel.addEventListener('click', () => this.closeReplyForm())
    reply.append(cancel)

    this.element.append(reply)
  }

  closeReplyForm() {
    if (this.replyButton) this.replyButton.hidden = false
    this.replyForm?.destroy()
  }
}
