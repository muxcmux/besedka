RSpec.describe 'Editing a comment with bad data' do
  let(:comment) { post('/api/comment', { site: 'test', path: '/', payload: { body: 'a comment' } }) }
  let(:token) { JSON.parse(comment.body, symbolize_names: true)[:token] }

  before do
    add_site('test', private: false, anonymous: true, moderated: false)
  end

  let(:response) { put("/api/comment/1", req) }

  context 'without site' do
    let(:req) { { path: '/' } }
    it 'returns serialization errors' do
      expect(response.status).to eq 422
      expect(response.body).to match(/missing field `site`/)
    end
  end

  context 'without a path' do
    let(:req) { { site: 'test' } }
    it 'returns serialization errors' do
      expect(response.status).to eq 422
      expect(response.body).to match(/missing field `path`/)
    end
  end

  context 'without a payload' do
    let(:req) { { site: 'test', path: '/' } }
    it 'returns errors' do
      expect(response.status).to eq 422
      expect(response.body).to match(/Payload can't be blank/)
    end
  end

  context 'with a missing body' do
    let(:req) { { site: 'test', path: '/', payload: {} } }
    it 'return serialization error' do
      expect(response.status).to eq 422
      expect(response.body).to match(/missing field `body`/)
    end
  end

  context 'with a blank body' do
    let(:req) { { site: 'test', path: '/', payload: { body: "" } } }
    it 'returns errors' do
      expect(response.status).to eq 422
      expect(response.body).to match(/Comment can't be blank/)
    end
  end

  context 'a comment which does not exist' do
    let(:req) { { site: 'test', path: '/', payload: { body: "updated comment" } } }
    let(:response) { put("/api/comment/42", req) }
    it 'returns not found' do
      token
      expect(response.status).to eq 404
    end
  end

  context 'without a token and not a moderator' do
    let(:req) { { site: 'test', path: '/', payload: { body: "updated comment" } } }
    it 'returns errors' do
      token
      expect(response.status).to eq 403
    end
  end
end

RSpec.describe 'Editing a comment with good data' do
  let(:comment) { post('/api/comment', { site: 'test', path: '/', payload: { body: 'a comment' } }) }
  let(:token) { JSON.parse(comment.body, symbolize_names: true)[:token] }

  let(:response) { put("/api/comment/1", req) }
  let(:json) { JSON.parse(response.body, symbolize_names: true) }
  let(:site) { add_site('test', private: false, anonymous: true, moderated: false) }

  before do
    site
    token
  end

  context 'with a valid token' do
    let(:req) { { site: 'test', path: '/', payload: { token:, body: 'updated *comment*' } } }

    it 'updates the comment' do
      expect(response.status).to eq(200)
      expect(json).to match(
        hash_including(
          comment: hash_including(
            body: 'updated *comment*',
            html_body: '<p>updated <em>comment</em></p>',
            edited: true
          )
        )
      )
    end
  end

  context 'a moderator' do
    let(:s) { sign({ name: 'moderator', moderator: true }, site) }
    let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'updated *comment*' } } }

    it 'updates the comment' do
      expect(response.status).to eq(200)
      expect(json).to match(
        hash_including(
          comment: hash_including(
            body: 'updated *comment*',
            html_body: '<p>updated <em>comment</em></p>',
            edited: true
          )
        )
      )
    end
  end
end
