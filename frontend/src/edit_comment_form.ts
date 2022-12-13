import NewCommentForm from "./new_comment_form"
import { createButton } from "./utils"

export default class EditCommentForm extends NewCommentForm {
  commentRecord?: CommentRecord
  cancel?: HTMLButtonElement
  onCancel: Function

  constructor(
    element: HTMLFormElement,
    comment: CommentRecord,
    callback: (response: PostCommentResponse) => void,
    onCancel: () => void
  ) {
    super(element, callback)
    this.onCancel = onCancel
    this.setComment(comment)
  }

  storageKey(): string {
    return `__besedka_edit_comment_draft_${this.commentRecord!.id}`
  }

  init() {
    this.button = createButton('Update comment', 'post-comment-button')
  }

  initUi() {
    this.cancel = createButton('Cancel', 'cancel-editing')
    this.cancel.setAttribute('type', 'button')
    this.cancel.addEventListener('click', () => {
      this.destroy()
      this.onCancel()
    })
    this.element.append(this.body, this.button, this.cancel, this.message)
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
