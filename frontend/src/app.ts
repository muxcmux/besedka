import {ApiRequest, Comment, CommentsResponse, Config, User} from "./types"
import CommentsContainer from "./comments_container"
import {post, reviver} from "./utils"
import NewComment from "./new_comment"

export default class App {
  container: HTMLElement
  // @ts-ignore strictPropertyInitialization
  config: Config
  req: ApiRequest
  user: User
  newComment: NewComment
  comments: CommentsContainer

  constructor(container: HTMLElement, config: ApiRequest, user: User) {
    this.container = container
    this.req = config
    this.user = user

    this.initUi()
    this.comments = new CommentsContainer(document.getElementById('besedka-comments')!, this)

    this.newComment = new NewComment(document.getElementById('besedka-new-comment')!, this)
    this.loadComments()
  }

  initUi() {
    this.container.innerHTML = `
      <div id="besedka-moderator-controls"></div>
      <div id="besedka-message"></div>
      <div id="besedka-new-comment"></div>
      <ul id="besedka-comments"></ul>
    `
  }

  error(msg: string) {
    document.getElementById('besedka-message')!.innerHTML = `
      <div class="besedka-error">${msg}</li>
    `
  }

  message(msg: string) {
    document.getElementById('besedka-message')!.innerText = msg
  }

  setToken(token: string) {
    window.localStorage.setItem('__besedka_token', token)
  }

  getToken(): string | null {
    return window.localStorage.getItem('__besedka_token')
  }

  setConfig(config: Config) {
    this.config = config
  }

  async loadComments() {
    try {
      const response = await post('/api/comments', Object.assign({
        payload: { token: this.getToken() }
      }, this.req))

      if (response.status == 404) {
        this.message("There are no comments yet. Be the first one to post!")
      } else {
        const comments = JSON.parse(await response.text(), reviver) as CommentsResponse
        this.setConfig(comments.site)
        if (comments.total) {
          this.renderComments(comments)
        } else {
          this.message("There are no comments yet. Be the first one to post!")
        }
      }
    } catch {
      this.error("Comments cannot be displayed at the moment")
    }
  }

  renderComments(data: CommentsResponse) {
    data.comments.forEach(c => {
      this.comments.add(c as Comment)
    })
  }
}
