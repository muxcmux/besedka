import EditCommentForm from "./edit_comment_form"
import NewCommentForm from "./new_comment_form"
import { createButton, createElement, getToken, request, timeago } from "./utils"

const TIME_TO_EDIT = 3 * 60

export default class Comment {
  editing = false
  onApprove: (instance: Comment) => void
  onDelete: (instance: Comment) => void
  comment: CommentRecord
  replyForm?: NewCommentForm<PostCommentResponse>
  replyButton?: HTMLButtonElement

  replies = createElement('ol', 'replies')
  element = createElement('li', 'comment')
  avatar  = createElement('div', 'avatar no-avatar')
  author  = createElement('div', 'comment-author')
  date    = createElement<HTMLTimeElement>('time', 'comment-timestamp')
  body    = createElement('div', 'comment-body')

  constructor(comment: CommentRecord) {
    this.onApprove = () => {}
    this.onDelete = (instance) => {
      if (instance.isReply()){
        if (instance.element.parentElement!.querySelectorAll('li').length == 1) {
          instance.element.parentElement!.parentElement!.classList.remove('besedka-has-replies');
        }
      } else {
        window.__besedka.updateCount(window.__besedka.commentCount - 1)
      }
    }

    this.comment = comment
    this.buildComment()

    if (window.__besedka.user.moderator || (this.comment.owned && this.withinEditingPeriod())) {
      this.element.append(this.createEditControls())
      this.element.append(this.createDeleteButton())
    }

    if (window.__besedka.user.moderator && !this.comment.reviewed) {
      this.element.append(this.createApproveButton())
    }

    this.buildReplies()

    if (this.canReply()) this.element.append(this.createReplyButton())
  }

  secondsSinceCreated(): number {
    const created = this.comment.created_at.getTime()
    const now = new Date().getTime()
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
    return !this.comment.parent_id && !window.__besedka.config?.locked
  }

  buildComment() {
    const { created_at, html_body, name, reviewed, owned, edited, op, moderator, replies } = this.comment

    if (!reviewed) this.element.classList.add('besedka-unreviewed-comment')
    if (owned) this.element.classList.add('besedka-owned-comment')
    if (edited) this.element.classList.add('besedka-edited-comment')
    if (moderator) this.element.classList.add('besedka-moderator-comment')
    if (op) this.element.classList.add('besedka-op-comment')
    if (replies?.length) this.element.classList.add('besedka-has-replies')

    this.author.textContent = name
    const localTimeString = created_at.toLocaleString(navigator.language, { dateStyle: "medium", timeStyle: "short" })
    this.date.setAttribute('datetime', localTimeString)
    this.date.setAttribute('title', localTimeString)
    this.date.textContent = timeago(created_at)
    this.body.innerHTML = html_body

    if (this.comment.avatar) {
      this.avatar.classList.remove('besedka-no-avatar')
      this.avatar.append(createElement<HTMLImageElement>('img', '', { src: this.comment.avatar, loading: 'lazy' }))
    }
    this.element.append(this.avatar, this.body, this.author, this.date)
  }

  createReplyButton(): HTMLButtonElement {
    this.replyButton = createButton('Reply', 'add-reply', { title: 'Reply' })
    this.replyButton.addEventListener('click', () => this.openReplyForm())
    return this.replyButton
  }

  createApproveButton(): HTMLButtonElement {
    const button = createButton('Approve', 'approve-comment', { title: 'Approve' })
    button.addEventListener('click', async () => {
      const { status } = await request(`/api/comment/${this.comment.id}`, window.__besedka.req, 'PATCH')
      if (status == 200) {
        this.element.classList.remove('besedka-unreviewed-comment')
        button.remove()
        this.onApprove(this)
      }
    })
    return button
  }

  url(): string {
    return `/api/comment/${this.comment.id}`
  }

  createDeleteButton(): HTMLButtonElement {
    const button = createButton('Delete', 'delete-comment', { title: 'Delete' })
    button.addEventListener('click', async () => {
      if (confirm("There's no undo. Proceed?")) {
        const { status } = await request(this.url(), Object.assign({
          payload: getToken()
        }, window.__besedka.req), 'DELETE')
        if (status == 200) this.destroy()
      }
    })

    if (!window.__besedka.user.moderator) this.expireControl(button)
    return button
  }

  createEditControls(): HTMLButtonElement {
    const button = createButton('Edit', 'edit-comment', { title: 'Edit' })
    button.addEventListener('click', () => {
      this.element.classList.add('besedka-editing-comment')
      const form = createElement<HTMLFormElement>('form', 'edit-comment-form')
      const editForm = new EditCommentForm<UpdateCommentResponse>(form, this.comment, ({ body, html_body }) => {
        this.comment.html_body = html_body
        this.comment.body = body
        this.body.innerHTML = html_body
        this.element.classList.add('besedka-edited-comment')
        this.element.classList.remove('besedka-editing-comment')
        editForm.destroy()
      }, () => {
        this.element.classList.remove('besedka-editing-comment')
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

  openReplyForm() {
    const reply = createElement<HTMLFormElement>('form', 'new-reply')

    this.replyForm = new NewCommentForm<PostCommentResponse>(reply, ({ comment }) => {
      this.replies.append(new Comment(comment).element)
      this.element.classList.add('besedka-has-replies')
      this.closeReplyForm()
      this.element.classList.remove('besedka-replying')
    }, this.comment.id)

    const cancel = createButton('Cancel', 'cancel-reply', { title: 'Cancel' })
    cancel.addEventListener('click', (e) => {
      e.preventDefault()
      this.closeReplyForm()
    })
    reply.append(cancel)

    this.element.append(reply)

    this.element.classList.add('besedka-replying')
    this.replyForm.body.focus()
  }

  closeReplyForm() {
    this.element.classList.remove('besedka-replying')
    this.replyForm?.destroy()
  }

  isReply() {
    return this.element.parentElement?.classList.contains('besedka-replies')
  }

  destroy() {
    this.onDelete(this)
    this.element.remove()
  }
}
