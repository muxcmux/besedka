def get_replies
  JSON.parse(post('/api/comments/1', { site: 'test', path: '/' }).body, symbolize_names: true)
end

RSpec.describe 'Requesting replies for a comment which does not exist' do
  it 'returns not found' do
    add_site('test', private: false, anonymous: true, moderated: false)
    response = post("/api/comments/42", { site: 'test', path: '/' })
    expect(response.status).to eq 404
    expect(response.body).to match(/Resource not found/)
  end
end

RSpec.describe 'Requesting replies for wrong site' do
  it 'returns a bad response' do
    add_site('test', private: false, anonymous: true, moderated: false)
    post('/api/comment', { site: 'test', path: '/', payload: { body: "hello world" } })
    response = post("/api/comments/1", { site: 'demo', path: '/' })
    expect(response.status).to eq 400
    expect(response.body).to match(/No configuration found/)
  end
end

RSpec.describe 'Multiple pages of replies' do
  before do
    add_site('test', private: false, anonymous: true, moderated: false)

    post('/api/comment', { site: 'test', path: '/', payload: { body: "hello world" } })
    5.times do |i|
      post(
        '/api/comment/1',
        { site: 'test', path: '/', payload: { body: "hello reply #{i}" } }
      )
    end
  end

  it 'displays a pages of replies with the newest first and a link to the next page' do
    response = get_replies

    expect(response).to match(
      hash_including(
        replies: [
          hash_including(id: 2, name: 'Anonymous', body: 'hello reply 0'),
          hash_including(id: 3, name: 'Anonymous', body: 'hello reply 1')
        ]
      )
    )

    expect(response[:cursor]).to_not be_nil

    second_page = JSON.parse(post("/api/comments/1?cursor=#{response[:cursor]}", { site: 'test', path: '/' }).body, symbolize_names: true)

    expect(second_page).to match(
      hash_including(
        replies: [
          hash_including(id: 4, name: 'Anonymous', body: 'hello reply 2'),
          hash_including(id: 5, name: 'Anonymous', body: 'hello reply 3')
        ]
      )
    )

    expect(second_page[:cursor]).to_not be_nil

    third_page = JSON.parse(post("/api/comments/1?cursor=#{second_page[:cursor]}", { site: 'test', path: '/' }).body, symbolize_names: true)

    expect(third_page).to match(
      hash_including(
        replies: [
          hash_including(id: 6, name: 'Anonymous', body: 'hello reply 4')
        ],
        cursor: nil
      )
    )
  end
end

RSpec.describe 'Listing replies from protected site' do
  before do
    secret = add_site('test', private: true, anonymous: true, moderated: false)
    @user, @signature = sign({ name: 'some user', moderator: false }, secret)

    post(
      '/api/comment',
      { site: 'test', path: '/', user: @user, signature: @signature, payload: { body: 'comment' } }
    )

    post(
      '/api/comment/1',
      { site: 'test', path: '/', user: @user, signature: @signature, payload: { body: 'reply' } }
    )
  end

  it 'displays the replies to a signed user' do
    response = JSON.parse(post("/api/comments/1", { site: 'test', path: '/', user: @user, signature: @signature }).body, symbolize_names: true)

    expect(response).to match(
      replies: [
        hash_including(id: 2, name: 'some user', body: 'reply')
      ],
      cursor: nil
    )
  end

  it 'does not display comments to non signed user' do
    response = post("/api/comments/1", { site: 'test', path: '/' })
    expect(response.status).to eq 401
  end
end

RSpec.describe 'Filtering replies' do
  let(:site) { add_site('test', private: false, anonymous: true, moderated: true) }
  let(:s) { sign({ name: 'moderator', moderator: true }, site) }

  let(:token) do
    JSON.parse(
      post('/api/comment/1', { site: 'test', path: '/', payload: { body: 'Another unreviewed reply' } }).body,
      symbolize_names: true
    )[:token]
  end

  before do
    post('/api/comment', { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'Reviewed comment' } })

    post('/api/comment/1', { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'Reviewed reply' } })
    post('/api/comment/1', { site: 'test', path: '/', payload: { body: 'Unreviewed reply' } })
    token
  end

  context 'without token' do
    it 'only displays reviewed comments' do
      expect(get_replies).to match(
        hash_including(
          replies: [
            hash_including(id: 2, name: 'moderator', body: 'Reviewed reply', reviewed: true)
          ],
          cursor: nil
        )
      )
    end
  end

  context 'with token' do
    it 'displays reviewed + own replies' do
      comments = JSON.parse(
        post('/api/comments/1', { site: 'test', path: '/', payload: { token: } }).body,
        symbolize_names: true
      )

      expect(comments).to match(
        hash_including(
          replies: [
            hash_including(id: 2, name: 'moderator', body: 'Reviewed reply', reviewed: true),
            hash_including(id: 4, name: 'Anonymous', body: 'Another unreviewed reply', reviewed: false)
          ],
          cursor: nil
        )
      )
    end
  end

  context 'a signed moderator' do
    it 'is able to see all replies' do
      comments = JSON.parse(
        post('/api/comments/1', { site: 'test', path: '/', user: s.first, signature: s.last }).body,
        symbolize_names: true
      )

      expect(comments).to match(
        hash_including(
          replies: [
            hash_including(id: 2, name: 'moderator', body: 'Reviewed reply', reviewed: true),
            hash_including(id: 3, name: 'Anonymous', body: 'Unreviewed reply', reviewed: false),
            hash_including(id: 4, name: 'Anonymous', body: 'Another unreviewed reply', reviewed: false)
          ],
          cursor: nil
        )
      )
    end
  end
end
