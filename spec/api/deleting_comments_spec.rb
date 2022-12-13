RSpec.describe 'Deleting a comment' do
  let(:comment) { post('/api/comment', { site: 'test', path: '/', payload: { body: 'a comment' } }) }
  let(:token) { JSON.parse(comment.body, symbolize_names: true)[:token] }
  let(:site) { add_site('test', private: false, anonymous: true, moderated: false) }

  before do
    site
    token
  end

  let(:response) { delete("/api/comment/1", req) }

  context 'a comment that does not exist' do
    let(:req) { { site: 'test', path: '/' } }
    let(:response) { delete("/api/comment/42", req) }
    it 'returns not found' do
      expect(response.status).to eq 404
    end
  end

  context 'without a token' do
    let(:req) { { site: 'test', path: '/' } }

    it 'returns forbidden' do
      expect(response.status).to eq 403
    end
  end

  context 'with an incorrect token' do
    let(:req) { { site: 'test', path: '/', payload: '123' } }

    it 'returns an error' do
      expect(response.status).to eq 422
    end
  end

  context 'with a correct token' do
    let(:req) { { site: 'test', path: '/', payload: token } }
    it 'deletes the comment' do
      expect(response.status).to eq 200
    end
  end

  context 'a moderator without token' do
    let(:s) { sign({ name: 'moderator', moderator: true }, site) }
    let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last } }

    it 'is allowed to delete' do
      expect(response.status).to eq 200
    end
  end
end
