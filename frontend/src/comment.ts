import EditCommentForm from "./edit_comment_form"
import NewCommentForm from "./new_comment_form"
import { createButton, createElement, getToken, request } from "./utils"

const TIME_TO_EDIT = 3 * 60

export default class Comment {
  comment: CommentRecord
  replyForm?: NewCommentForm
  replyButton?: HTMLButtonElement

  replies = createElement('ol', 'replies')
  element = createElement('li', 'comment')
  author  = createElement('div', 'comment-author')
  date    = createElement('div', 'comment-timestamp')
  body    = createElement('div', 'comment-body')

  constructor(comment: CommentRecord) {
    this.comment = comment
    this.buildComment(this.comment)

    if (window.__besedka.user.moderator && !this.comment.reviewed) {
      this.element.append(this.createApproveButton())
    }

    if (window.__besedka.user.moderator || (this.comment.owned && this.withinEditingPeriod())) {
      this.element.append(this.createEditControls())
      this.element.append(this.createDeleteButton())
    }

    this.buildReplies()

    if (this.canReply()) this.element.append(this.createReplyButton())
  }

  secondsSinceCreated(): number {
    const created = this.comment.created_at.getTime()
    const now = new Date(new Date().toUTCString()).getTime()
    return (now - created) / 1000
  }

  withinEditingPeriod(): boolean {
    return this.secondsSinceCreated() <= TIME_TO_EDIT
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

  buildComment({ created_at, html_body, name, reviewed, locked, owned, edited }: CommentRecord) {
    if (!reviewed) this.element.classList.add('besedka-unreviewed-comment')
    if (locked) this.element.classList.add('besedka-locked-comment')
    if (owned) this.element.classList.add('besedka-owned-comment')
    if (edited) this.element.classList.add('besedka-edited-comment')

    this.author.textContent = name
    this.date.textContent = created_at.toLocaleString(navigator.language, { dateStyle: "medium", timeStyle: "short" })
    this.body.innerHTML = html_body

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

  url(): string {
    return `/api/comment/${this.comment.id}`
  }

  createDeleteButton(): HTMLButtonElement {
    const button = createButton('Delete', 'delete-comment')
    button.addEventListener('click', async () => {
      if (confirm("There's no undo. Proceed?")) {
        const { status } = await request(this.url(), Object.assign({
          payload: getToken()
        }, window.__besedka.req), 'DELETE')
        if (status == 200) this.element.remove()
      }
    })

    if (!window.__besedka.user.moderator) this.expireControl(button)
    return button
  }

  createEditControls(): HTMLButtonElement {
    const button = createButton('Edit', 'edit-comment')
    button.addEventListener('click', () => {
      button.hidden = true
      this.body.hidden = true
      const form = createElement<HTMLFormElement>('form', 'edit-comment')
      const editForm = new EditCommentForm(form, this.comment, ({ comment }) => {
        this.comment.html_body = comment.html_body
        this.comment.body = comment.body
        this.body.innerHTML = comment.html_body
        this.body.hidden = false
        this.body.classList.add('besedka-edited-comment')
        editForm.destroy()
        button.hidden = false
      }, () => {
        this.body.hidden = false
        button.hidden = false
      })
      this.element.insertBefore(form, this.body)
    })

    if (!window.__besedka.user.moderator) this.expireControl(button)
    return button
  }

  expireControl(element: HTMLButtonElement) {
    setTimeout(() => {
      element.remove()
    }, (TIME_TO_EDIT - this.secondsSinceCreated()) * 1000)
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
