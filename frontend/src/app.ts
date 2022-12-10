import NewCommentForm from "./new_comment_form"
import Comment from "./comment"
import {getToken, message, request, setToken} from "./utils"
import ModeratorControls from "./moderator_controls"

export default class App {
  element: HTMLElement
  config: Config | null = null
  req: ApiRequest
  user: User
  // @ts-ignore strictPropertyInitialization
  comments: HTMLOListElement
  // @ts-ignore strictPropertyInitialization
  endOfComments: HTMLDivElement
  observer = new IntersectionObserver((entries) => {
    if (entries && entries[0].isIntersecting && !this.loading && this.cursor) this.loadComments()
  })

  // @ts-ignore strictPropertyInitialization
  observed: boolean
  // @ts-ignore strictPropertyInitialization
  loading: boolean
  // @ts-ignore strictPropertyInitialization
  cursor: string | null

  constructor(element: HTMLDivElement, config: ApiRequest, user: User) {
    this.element = element
    this.req = config
    this.user = user
  }

  async run() {
    this.element.innerHTML = ''
    this.observed = false
    this.loading = false
    this.cursor = null

    this.draw()
    await this.loadConfig()

    new ModeratorControls(document.getElementById('besedka-moderator-controls') as HTMLDivElement)

    if (this.config) await this.loadComments()

    if (this.config?.locked) {
      message('Leaving comments on this page has been disabled', 'info')
    } else {
      new NewCommentForm(document.getElementById('besedka-new-comment') as HTMLFormElement, ({ token, comment }) => {
        setToken(token)
        this.comments.prepend(new Comment(comment).element)
      })
    }
  }

  draw() {
    this.element.innerHTML = `
      <div id="besedka-moderator-controls"></div>
      <form id="besedka-new-comment"></form>
      <div id="besedka-message"></div>
      <ol id="besedka-comments"></ol>
      <div id="besedka-end-of-comments"></div>
    `
    this.comments = document.getElementById('besedka-comments') as HTMLOListElement
    this.endOfComments = document.getElementById('besedka-end-of-comments') as HTMLDivElement
  }

  commentUrl(): string {
    return this.cursor ? `/api/comments?cursor=${this.cursor}` : '/api/comments'
  }

  async loadComments() {
    this.loading = true
    try {
      const { status, json } = await request<CommentsResponse>(this.commentUrl(), Object.assign({
        payload: { token: getToken() }
      }, this.req))

      if (status == 404 || (json && json.total == 0)) {
        message("There are no comments yet. Be the first one to post!", "info")
      } else if (json) {
        this.renderComments(json.comments)
        this.cursor = json.cursor

        if (!this.observed) {
          this.observer.observe(this.endOfComments)
          this.observed = true
        }
      }
    } finally {
      this.loading = false
    }
  }

  async loadConfig() {
    const { json } = await request<Config>('/api/config', this.req)
    this.config = json
  }

  renderComments(comments: CommentRecord[]) {
    comments.forEach(c => this.comments.append(new Comment(c).element))
  }
}
