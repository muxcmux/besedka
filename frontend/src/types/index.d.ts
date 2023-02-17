export {};

import App from '../app'

declare global {
  interface Window {
    __besedka: App
  }

  interface CommentRecord {
    id: number
    parent_id?: number
    name: string
    html_body: string
    body: string
    avatar?: string
    reviewed: boolean
    created_at: Date
    updated_at: Date
    owned: boolean
    op: boolean
    moderator: boolean
    edited: boolean
    replies?: CommentRecord[]
    page_path?: string
    page_title?: string
  }

  interface CommentsResponse {
    total: number
    cursor: string | null
    comments: CommentRecord[]
  }

  interface ApiRequest {
    site: string
    path: string
    title: string
    user?: string
    signature?: string
    sid?: string
  }

  interface Config {
    anonymous: boolean
    moderated: boolean
    locked: boolean
  }

  interface CreateCommentRequest extends ApiRequest {
    payload?: {
      body: string
      name?: string
      token?: string
    }
  }

  interface PostCommentResponse {
    comment: CommentRecord
    token: string
  }

  interface UpdateCommentResponse {
    body: string
    html_body: string
  }

  interface User {
    name?: string
    moderator?: boolean
    avatar?: string
    op?: boolean
  }

  interface LoginResponse {
    name: string
    sid: string
    avatar?: string
    op?: boolean
  }
}
