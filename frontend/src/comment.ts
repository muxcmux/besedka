import NewCommentForm from "./new_comment_form"
import { createButton, createElement, request } from "./utils"

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

    if (window.__besedka.user.moderator && !this.comment.reviewed) {
      this.element.append(this.createApproveButton())
    }

    if (window.__besedka.user.moderator || (this.comment.owned && !this.comment.replies?.length)) {
      this.element.append(this.createDeleteButton())
    }

    if (window.__besedka.user.moderator || this.comment.owned) {
      this.element.append(this.createEditButton())
    }

    this.buildReplies()

    if (this.canReply()) this.element.append(this.createReplyButton())
  }

  buildReplies() {
    this.element.append(this.replies)

    if (this.comment.replies?.length) {
      this.comment.replies?.forEach(reply => {
        const nested = new Comment(reply)
        this.replies!.append(nested.element)
      })
    }
  }

  canReply() {
    return !this.comment.parent_id && !this.comment.locked && !window.__besedka.config?.locked
  }

  buildComment({ created_at, body, name, reviewed, locked, owned }: CommentRecord) {
    if (!reviewed) this.element.classList.add('besedka-unreviewed-comment')
    if (locked) this.element.classList.add('besedka-locked-comment')
    if (owned) this.element.classList.add('besedka-owned-comment')

    this.author.textContent = name
    this.date.textContent = created_at.toLocaleString(navigator.language, { dateStyle: "medium", timeStyle: "short" })
    this.body.textContent = body

    this.element.append(this.author, this.date, this.body)
  }

  createReplyButton(): HTMLButtonElement {
    this.replyButton = createButton('Reply', 'add-reply')
    this.replyButton.addEventListener('click', e => this.openReplyForm(e))
    return this.replyButton
  }

  createApproveButton(): HTMLButtonElement {
    const button = createButton('Approve', 'approve-comment')
    button.addEventListener('click', async () => {
      const { status } = await request(`/api/comment/${this.comment.id}`, window.__besedka.req, 'PATCH')
      if (status == 200) {
        this.element.classList.remove('besedka-unreviewed-comment')
        button.remove()
      }
    })
    return button
  }

  createDeleteButton(): HTMLButtonElement {
    const button = createButton('Delete', 'delete-comment')
    button.addEventListener('click', async () => {
      if (confirm("There's no undo. Proceed?")) {
        // await request(`/api/comment/${this.comment.id}`, window.)
        this.element.remove()
      }
    })
    return button
  }

  createEditButton(): HTMLButtonElement {
    const button = createButton('Edit', 'edit-comment')
    return button
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
