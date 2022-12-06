RSpec.describe 'Posting a reply with bad data' do
  before do
    add_site('test', private: false, anonymous: true, moderated: false)
    post("/api/comment", { site: 'test', path: '/', payload: { body: "hello" } })
  end

  let(:req) { { site: 'demo', path: '/' } }
  let(:comment_id) { 1 }
  let(:response) { post("/api/comment/#{comment_id}", req) }

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

  context 'for missing comment' do
    let(:comment_id) { 2 }
    let(:req) { { site: 'test', path: '/', payload: { body: "reply!" } } }
    it 'returns not found' do
      expect(response.status).to eq 404
    end
  end

  context 'for a non-root comment' do
    let(:comment_id) { 2 }
    let(:req) { { site: 'test', path: '/', payload: { body: "reply!" } } }
    before do
      post("/api/comment/1", { site: 'test', path: '/', payload: { body: "hello" } })
    end

    it 'returns not found' do
      expect(response.status).to eq 404
    end
  end
end

RSpec.describe 'Anonymous replying to comments' do
  let(:req) { { site: 'test', path: '/', payload: { body: 'a reply' } } }
  let(:response) { post("/api/comment/1", req) }
  let(:json) { JSON.parse(response.body, symbolize_names: true) }

  context 'is allowed' do
    before do
      add_site('test', private: false, anonymous: true, moderated: false)
      post("/api/comment", { site: 'test', path: '/', payload: { body: "a comment" } })
    end

    it 'allows posting and assigns Anonymous as name' do
      expect(json[:comment]).to match(hash_including(id: 2, name: 'Anonymous', body: 'a reply'))
      expect(json[:token]).to_not be_nil
    end
  end

  context 'is not allowed' do
    let(:site) { add_site('test', private: false, anonymous: false, moderated: false) }
    let(:s) { sign({ name: 'some user', moderator: false }, site) }

    before do
      post("/api/comment", { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: "a comment" } })
    end

    context 'an anonymous user' do
      it 'is not allowed to post' do
        expect(response.status).to eq 401
      end
    end

    context 'a signed user' do
      let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'a reply' } } }

      it 'is allowed to post' do
        expect(json[:comment]).to match(hash_including(id: 2, name: 'some user', body: 'a reply'))
      end
    end

    context 'a logged in moderator' do
      let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'a reply' } } }
      let(:s) { sign({ name: 'moderator', moderator: true }, site) }

      it 'is allowed to post' do
        expect(json[:comment]).to match(hash_including(id: 2, name: 'moderator', body: 'a reply'))
      end
    end
  end
end

RSpec.describe 'Moderation' do
  let(:req) { { site: 'test', path: '/', payload: { body: 'reply' } } }
  let(:response) { post("/api/comment/1", req) }
  let(:json) { JSON.parse(response.body, symbolize_names: true) }
  let(:moderated) { false }
  let(:site) { add_site('test', private: false, anonymous: true, moderated:) }

  let(:a) { sign({ name: 'moderator', moderator: true }, site) }

  before do
    post("/api/comment", { site: 'test', path: '/', user: a.first, signature: a.last, payload: { body: 'a comment' } })
  end

  context 'disabled' do
    it 'allows replies straight away' do
      expect(json[:comment]).to match(hash_including(id: 2, name: 'Anonymous', body: 'reply', reviewed: true))
    end
  end

  context 'enabled' do
    let(:moderated) { true }

    it 'requires replies to be reviewed' do
      expect(json[:comment]).to match(hash_including(id: 2, name: 'Anonymous', body: 'reply', reviewed: false))
    end

    context 'a signed user reply' do
      let(:s) { sign({ name: 'some user', moderator: false }, site) }
      let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'reply' } } }

      it 'must be reviewed' do
        expect(json[:comment]).to match(hash_including(id: 2, name: 'some user', body: 'reply', reviewed: false))
      end
    end

    context 'a signed moderator reply' do
      let(:req) { { site: 'test', path: '/', user: a.first, signature: a.last, payload: { body: 'reply' } } }

      it 'must be reviewed' do
        expect(json[:comment]).to match(hash_including(id: 2, name: 'moderator', body: 'reply', reviewed: true))
      end
    end
  end
end

RSpec.describe 'Responder name' do
  let(:req) { { site: 'test', path: '/', payload: { name: 'Responder', body: 'reply' } } }
  let(:response) { post("/api/comment/1", req) }
  let(:json) { JSON.parse(response.body, symbolize_names: true) }
  let(:site) { add_site('test', private: false, anonymous: true, moderated: false) }

  before do
    site
    post("/api/comment", { site: 'test', path: '/', payload: { body: 'a comment' } })
  end


  context 'an anonymous user' do
    it 'can set a name' do
      expect(json[:comment]).to match(hash_including(id: 2, name: 'Responder', body: 'reply'))
    end

    context 'without a name' do
      let(:req) { { site: 'test', path: '/', payload: { body: 'reply' } } }

      it 'defaults to Anonymous' do
        expect(json[:comment]).to match(hash_including(id: 2, name: 'Anonymous', body: 'reply'))
      end
    end

    context 'with a blank name' do
      let(:req) { { site: 'test', path: '/', payload: { name: '   ', body: 'reply' } } }

      it 'defaults to Anonymous' do
        expect(json[:comment]).to match(hash_including(id: 2, name: 'Anonymous', body: 'reply'))
      end
    end
  end

  context 'a signed user' do
    let(:s) { sign({ name: 'signed user', moderator: false }, site) }
    let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'reply' } } }

    it 'cannot set their name' do
      expect(json[:comment]).to match(hash_including(id: 2, name: 'signed user', body: 'reply'))
    end
  end

  context 'a moderator' do
    let(:s) { sign({ name: 'moderator', moderator: true }, site) }
    let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'reply' } } }

    it 'cannot set their name' do
      expect(json[:comment]).to match(hash_including(id: 2, name: 'moderator', body: 'reply'))
    end
  end
end
