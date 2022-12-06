import {ApiRequest, Comment, CommentsResponse, Config, User} from "./types"
import CommentsContainer from "./comments_container"
import {getToken, message, post} from "./utils"
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
      <div id="besedka-new-comment"></div>
      <div id="besedka-message"></div>
      <ul id="besedka-comments"></ul>
    `
  }

  setConfig(config: Config) {
    this.config = config
  }

  async loadComments() {
    const { status, json } = await post<CommentsResponse>('/api/comments', Object.assign({
      payload: { token: getToken() }
    }, this.req))

    if (status == 404 || (json && json.total == 0)) {
      message("There are no comments yet. Be the first one to post!", "info")
    } else if (json) {
      this.setConfig(json.site)
      this.renderComments(json.comments)
    }
  }

  renderComments(comments: Comment[]) {
    comments.forEach(c => {
      this.comments.add(c as Comment)
    })
  }
}
