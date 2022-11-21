import {ConfigRequest, Comment, Comments} from "./types"
import CommentsContainer from "./comments_container"

export default class App {
  container: HTMLElement
  config: ConfigRequest
  host: string
  comments: CommentsContainer

  constructor(container: HTMLElement, config: ConfigRequest) {
    this.container = container
    this.config= config
    this.host = "http://besedka.com:6353"

    this.initUi()
    this.comments = new CommentsContainer(document.getElementById('besedka-comments')!)

    this.loadComments()
  }

  initUi() {
    this.container.innerHTML = `
      <div id="besedka-comment">
        <input type="text" placeholder="Your name" />
        <textarea placeholder="Leave a comment"></textarea>
        <button>Post</button>
      </div>
      <ul id="besedka-comments"></ul>
    `
  }

  error(msg: string) {
    document.getElementById('besedka-comments')!.innerHTML = `
      <li class="besedka-error">${msg}</li>
    `
  }

  errorFromStatus(status: number) {
    switch(status) {
      case 404: "No comments yet"
      case 401: "You need to be authorised to see the comments"
      case 403: "C"
    }
  }

  async loadComments() {
    try {
      const response = await fetch(`${this.host}/api/shitcomments`, {
        method: "POST",
        mode: "cors",
        body: JSON.stringify(this.config),
        headers: { "Content-Type": "application/json" }
      })

      if (!response.ok) this.errorFromStatus(response.status)

      this.renderComments(await response.json() as Comments)
    } catch {
      this.error("Comments cannot be displayed at the moment")
    }
  }

  renderComments(data: Comments) {
    data.comments.forEach(c => {
      this.comments.add(c as Comment)
    })
  }
}
