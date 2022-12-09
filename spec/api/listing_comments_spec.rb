def get_comments
  JSON.parse(post('/api/comments', { site: 'test', path: '/' }).body, symbolize_names: true)
end

RSpec.describe 'Single page of comments' do
  before do
    add_site('test', private: false, anonymous: true, moderated: false)

    5.times do |i|
      post(
        '/api/comment',
        { site: 'test', path: '/', payload: { body: "hello world #{i}" } }
      )
    end
  end

  it 'displays a list of comments' do
    expect(get_comments).to match(
      hash_including(
        comments: [
          hash_including(id: 5, name: 'Anonymous', body: 'hello world 4', thread: { cursor: nil, replies: [] }),
          hash_including(id: 4, name: 'Anonymous', body: 'hello world 3', thread: { cursor: nil, replies: [] }),
          hash_including(id: 3, name: 'Anonymous', body: 'hello world 2', thread: { cursor: nil, replies: [] }),
          hash_including(id: 2, name: 'Anonymous', body: 'hello world 1', thread: { cursor: nil, replies: [] }),
          hash_including(id: 1, name: 'Anonymous', body: 'hello world 0', thread: { cursor: nil, replies: [] })
        ],
        cursor: nil,
        total: 5
      )
    )
  end
end

RSpec.describe 'Multiple pages of comments' do
  before do
    add_site('test', private: false, anonymous: true, moderated: false)

    5.times do |i|
      post(
        '/api/comment',
        { site: 'test', path: '/', payload: { body: "hello world #{i}" } }
      )
    end
  end

  it 'displays a pages of comments with the newest on top and a links to the next page' do
    response = get_comments

    expect(response).to match(
      hash_including(
        comments: [
          hash_including(id: 5, name: 'Anonymous', body: 'hello world 4', thread: { cursor: nil, replies: [] }),
          hash_including(id: 4, name: 'Anonymous', body: 'hello world 3', thread: { cursor: nil, replies: [] })
        ],
        total: 5
      )
    )

    expect(response[:cursor]).to_not be_nil

    second_page = JSON.parse(post("/api/comments?cursor=#{response[:cursor]}", { site: 'test', path: '/' }).body, symbolize_names: true)

    expect(second_page).to match(
      hash_including(
        comments: [
          hash_including(id: 3, name: 'Anonymous', body: 'hello world 2', thread: { cursor: nil, replies: [] }),
          hash_including(id: 2, name: 'Anonymous', body: 'hello world 1', thread: { cursor: nil, replies: [] })
        ],
        total: 5
      )
    )

    expect(second_page[:cursor]).to_not be_nil

    third_page = JSON.parse(post("/api/comments?cursor=#{second_page[:cursor]}", { site: 'test', path: '/' }).body, symbolize_names: true)

    expect(third_page).to match(
      hash_including(
        comments: [
          hash_including(id: 1, name: 'Anonymous', body: 'hello world 0', thread: { cursor: nil, replies: [] })
        ],
        cursor: nil,
        total: 5
      )
    )
  end
end

RSpec.describe 'Two pages of comments and a thread' do
  before do
    add_site('test', private: false, anonymous: true, moderated: false)

    5.times do |i|
      post(
        '/api/comment',
        { site: 'test', path: '/', payload: { body: "hello world #{i}" } }
      )

      if i == 2
        3.times do |j|
          post(
            "/api/comment/#{i + 1}",
            { site: 'test', path: '/', payload: { body: "Reply #{j}" } }
          )
        end
      end
    end
  end

  it 'displays a list of comments and the first page of replies on the second page' do
    response = get_comments
    expect(response).to match(
      hash_including(
        comments: [
          hash_including(id: 8, name: 'Anonymous', body: 'hello world 4', thread: { cursor: nil, replies: [] }),
          hash_including(id: 7, name: 'Anonymous', body: 'hello world 3', thread: { cursor: nil, replies: [] })
        ],
        total: 5
      )
    )
    expect(response[:cursor]).to_not be_nil

    second_page = JSON.parse(post("/api/comments?cursor=#{response[:cursor]}", { site: 'test', path: '/' }).body, symbolize_names: true)
    expect(second_page).to match(
      hash_including(
        comments: [
          hash_including(id: 3, name: 'Anonymous', body: 'hello world 2', thread:
            hash_including(replies: [
              hash_including(id: 4, name: 'Anonymous', body: 'Reply 0'),
              hash_including(id: 5, name: 'Anonymous', body: 'Reply 1')
            ])
          ),
          hash_including(id: 2, name: 'Anonymous', body: 'hello world 1', thread: { cursor: nil, replies: [] })
        ],
        total: 5
      )
    )
  end

  it 'displays the replies from the replies endpoint' do
    thread = JSON.parse(post('/api/comments/3', { site: 'test', path: '/' }).body, symbolize_names: true)
    expect(thread).to match(
      hash_including(
        replies: [
          hash_including(id: 4, name: 'Anonymous', body: 'Reply 0'),
          hash_including(id: 5, name: 'Anonymous', body: 'Reply 1')
        ]
      )
    )
    expect(thread[:cursor]).to_not be_nil

    second_page = JSON.parse(post("/api/comments/3?cursor=#{thread[:cursor]}", { site: 'test', path: '/' }).body, symbolize_names: true)
    expect(second_page).to match(
      hash_including(
        replies: [
          hash_including(id: 6, name: 'Anonymous', body: 'Reply 2')
        ],
        cursor: nil
      )
    )
  end
end

RSpec.describe 'Listing comments from protected site' do
  before do
    secret = add_site('test', private: true, anonymous: true, moderated: false)
    @user, @signature = sign({ name: 'some user', moderator: false }, secret)

    post(
      '/api/comment',
      { site: 'test', path: '/', user: @user, signature: @signature, payload: { body: 'comment' } }
    )
  end

  it 'displays the comments to a signed user' do
    response = JSON.parse(post("/api/comments", { site: 'test', path: '/', user: @user, signature: @signature }).body, symbolize_names: true)

    expect(response).to match(
      hash_including(
        comments: [
          hash_including(id: 1, name: 'some user', body: 'comment')
        ],
        total: 1,
        cursor: nil
      )
    )
  end

  it 'does not display comments to non signed user' do
    response = post("/api/comments", { site: 'test', path: '/' })
    expect(response.status).to eq 401
  end
end

RSpec.describe 'Requesting comments for a site which does not exist' do
  it 'returns a bad response' do
    add_site('test', private: false, anonymous: true, moderated: false)
    response = post("/api/comments", { site: 'demo', path: '/' })
    expect(response.status).to eq 400
    expect(response.body).to match(/No configuration found/)
  end
end

RSpec.describe 'Requesting comments for a page that does not exist' do
  it 'returns not found' do
    add_site('test', private: false, anonymous: true, moderated: false)
    response = post("/api/comments", { site: 'test', path: '/blog' })
    expect(response.status).to eq 404
    expect(response.body).to match(/Resource not found/)
  end
end

RSpec.describe 'Filtering comments' do
  let(:site) { add_site('test', private: false, anonymous: true, moderated: true) }
  let(:s) { sign({ name: 'moderator', moderator: true }, site) }
  let(:token) do
    JSON.parse(
      post('/api/comment', { site: 'test', path: '/', payload: { body: 'Another unreviewed comment' } }).body,
      symbolize_names: true
    )[:token]
  end

  before do
    post('/api/comment', { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'Reviewed comment' } })
    post('/api/comment', { site: 'test', path: '/', payload: { body: 'Unreviewed comment' } })
    token

    post('/api/comment', { site: 'test', path: '/blog', user: s.first, signature: s.last, payload: { body: 'Reviewed blog comment' } })
  end

  context 'without token' do
    it 'only displays reviewed comments' do
      expect(get_comments).to match(
        hash_including(
          comments: [
            hash_including(id: 1, name: 'moderator', body: 'Reviewed comment', reviewed: true)
          ],
          total: 1
        )
      )
    end
  end

  context 'with token' do
    it 'displays reviewed + own comments' do
      comments = JSON.parse(
        post('/api/comments', { site: 'test', path: '/', payload: { token: } }).body,
        symbolize_names: true
      )

      expect(comments).to match(
        hash_including(
          comments: [
            hash_including(id: 3, name: 'Anonymous', body: 'Another unreviewed comment', reviewed: false),
            hash_including(id: 1, name: 'moderator', body: 'Reviewed comment', reviewed: true)
          ],
          total: 2
        )
      )
    end
  end

  context 'a signed moderator' do
    it 'is able to see all comments' do
      comments = JSON.parse(
        post('/api/comments', { site: 'test', path: '/', user: s.first, signature: s.last }).body,
        symbolize_names: true
      )

      expect(comments).to match(
        hash_including(
          comments: [
            hash_including(id: 3, name: 'Anonymous', body: 'Another unreviewed comment', reviewed: false),
            hash_including(id: 2, name: 'Anonymous', body: 'Unreviewed comment', reviewed: false),
            hash_including(id: 1, name: 'moderator', body: 'Reviewed comment', reviewed: true)
          ],
          total: 3
        )
      )
    end
  end

  context 'with nested replies' do
    let(:reply_token) do
      JSON.parse(
        post('/api/comment/1', { site: 'test', path: '/', payload: { body: 'Another unreviewed reply' } }).body,
        symbolize_names: true
      )[:token]
    end

    before do
      post('/api/comment/1', { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'Reviewed reply' } })
      post('/api/comment/1', { site: 'test', path: '/', payload: { body: 'Unreviewed reply' } })
    end

    context 'without token' do
      it 'only displays reviewed replies' do
        expect(get_comments).to match(
          hash_including(
            comments: [
              hash_including(id: 1, name: 'moderator', body: 'Reviewed comment', reviewed: true, thread: {
                cursor: nil,
                replies: [
                  hash_including(id: 5, name: 'moderator', body: 'Reviewed reply', reviewed: true)
                ]
              })
            ],
            total: 1
          )
        )
      end
    end

    context 'with token' do
      it 'displays reviewed + own replies' do
        comments = JSON.parse(
          post('/api/comments', { site: 'test', path: '/', payload: { token: reply_token } }).body,
          symbolize_names: true
        )

        expect(comments).to match(
          hash_including(
            comments: [
              hash_including(id: 1, name: 'moderator', body: 'Reviewed comment', reviewed: true, thread: {
                cursor: nil,
                replies: [
                  hash_including(id: 5, name: 'moderator', body: 'Reviewed reply', reviewed: true),
                  hash_including(id: 7, name: 'Anonymous', body: 'Another unreviewed reply', reviewed: false)
                ]
              })
            ],
            total: 1
          )
        )
      end
    end

    context 'a signed moderator' do
      it 'is able to see all replies' do
        comments = JSON.parse(
          post('/api/comments', { site: 'test', path: '/', user: s.first, signature: s.last }).body,
          symbolize_names: true
        )

        expect(comments).to match(
          hash_including(
            comments: [
              hash_including(id: 3, name: 'Anonymous', body: 'Another unreviewed comment', reviewed: false),
              hash_including(id: 2, name: 'Anonymous', body: 'Unreviewed comment', reviewed: false),
              hash_including(id: 1, name: 'moderator', body: 'Reviewed comment', reviewed: true, thread: {
                cursor: nil,
                replies: [
                  hash_including(id: 5, name: 'moderator', body: 'Reviewed reply', reviewed: true),
                  hash_including(id: 6, name: 'Anonymous', body: 'Unreviewed reply', reviewed: false)
                ]
              })
            ],
            total: 3
          )
        )
      end
    end
  end
end
