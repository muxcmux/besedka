export interface Thread {
  cursor?: string
  replies: Comment[]
}

export interface Comment {
  id: number
  parent_id?: number
  name: string
  body: string
  avatar?: string
  locked: boolean
  reviewed: boolean
  created_at: Date
  updated_at: Date
  thread: Thread
  token: string
}

export interface CommentsResponse {
  total: number
  cursor?: string
  comments: Comment[]
  site: Config
}

export interface ApiRequest {
  site: string
  path: string
  user?: string
  signature?: string
  sid?: string
}

export interface Config {
  private: boolean
  anonymous: boolean
  moderated: boolean
  comments_per_page: number
  replies_per_comment: number
  minutes_to_edit: number
  theme: string
}

export interface CreateCommentRequest extends ApiRequest {
  payload?: {
    body: string
    name?: string
    token?: string
  }
}

export interface PostCommentResponse {
  comment: Comment
  token: string
  site: Config
}

export interface User {
  name?: string
  moderator?: boolean
  avatar?: string
}
