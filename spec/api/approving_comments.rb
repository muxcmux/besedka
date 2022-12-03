RSpec.describe 'approving comments' do
  let(:site) { add_site('test', private: false, anonymous: true, moderated: true) }

  before do
    site
    post('/api/comment', { site: 'test', path: '/', payload: { body: "hello world" } })
  end

  context 'an anonymous user' do
    it 'is unable to approve comments' do
      response = patch('/api/comment/1', site: 'test', path: '/')
      expect(response.status).to eq(401)
    end
  end

  context 'a signed user' do
    let(:s) { sign({ name: 'user', moderator: false }, site) }

    it 'is unable to approve comment' do
      response = patch('/api/comment/1', site: 'test', path: '/', user: s.first, signature: s.last)
      expect(response.status).to eq(403)
    end
  end

  context 'a moderator' do
    let(:s) { sign({ name: 'moderator', moderator: true }, site) }

    it 'is able to approve comments' do
      response = patch('/api/comment/1', site: 'test', path: '/', user: s.first, signature: s.last)
      expect(response.status).to eq(200)
    end
  end
end
