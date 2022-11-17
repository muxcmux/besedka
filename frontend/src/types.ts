export interface Thread {
  cursor?: string;
  replies: Comment[];
}

export interface Comment {
  id: number;
  parent_id?: number;
  name: string;
  body: string;
  avatar?: string;
  replies_count: number;
  locked: boolean;
  created_at: Date;
  updated_at: Date;
  thread: Thread;
}

export interface Comments {
  total: number;
  cursor?: string;
  comments: Comment[];
}

export interface Config {
  site: string,
  path: string,
  private?: boolean,
  anonymous_comments?: boolean,
  moderated?: boolean,
  comments_per_page?: number,
  replies_per_comment?: number,
  minutes_to_edit?: number,
  theme?: string,
  user?: User
}

export interface ConfigRequest {
  site: string,
  path: string,
  config?: string,
  signature?: string
}

export interface User {
  id: string,
  username?: string,
  name?: string,
  moderator?: boolean,
  avatar: string
}
