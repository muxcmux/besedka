import NewCommentForm from "./new_comment_form"
import { createButton } from "./utils"

export default class EditCommentForm<R> extends NewCommentForm<R> {
  commentRecord?: CommentRecord
  cancel?: HTMLButtonElement
  onCancel: Function

  constructor(
    element: HTMLFormElement,
    comment: CommentRecord,
    callback: (response: R) => void,
    onCancel: () => void
  ) {
    super(element, callback)
    this.onCancel = onCancel
    this.setComment(comment)
    window.requestAnimationFrame(() => this.body.focus())
  }

  storageKey(): string {
    return `__besedka_edit_comment_draft_${this.commentRecord!.id}`
  }

  init() {
    // no op
  }

  initUi() {
    this.cancel = createButton('Cancel', 'cancel-editing', { type: 'button' })
    this.cancel.addEventListener('click', () => {
      this.destroy()
      this.onCancel()
    })

    this.element.append(this.body, this.previewBody, this.button, this.cancel)
  }

  setComment(comment: CommentRecord) {
    this.commentRecord = comment
    const val = window.localStorage.getItem(this.storageKey())
    this.body.value = val || this.commentRecord.body
  }

  url(): string {
    return `/api/comment/${this.commentRecord!.id}`
  }

  method(): string { return 'PUT' }
}
