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
    body: string
    avatar?: string
    locked: boolean
    reviewed: boolean
    created_at: Date
    updated_at: Date
    owned: boolean
    replies?: CommentRecord[]
  }

  interface CommentsResponse {
    total: number
    cursor: string | null
    comments: CommentRecord[]
  }

  interface ApiRequest {
    site: string
    path: string
    user?: string
    signature?: string
    sid?: string
  }

  interface Config {
    anonymous: boolean
    moderated: boolean
    locked: boolean
    theme: string
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

  interface User {
    name?: string
    moderator?: boolean
    avatar?: string
  }

  interface LoginResponse {
    name: string,
    sid: string,
    avatar?: string
  }
}

