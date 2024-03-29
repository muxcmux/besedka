RSpec.describe 'Posting a comment with bad data' do
  before do
    add_site('test', private: false, anonymous: true, moderated: false)
  end

  let(:response) { post("/api/comment", req) }

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

  context 'with an incorrect site' do
    let(:req) { { site: 'demo', path: '/', payload: { body: "hello" } } }
    it 'returns errors' do
      expect(response.status).to eq 400
      expect(response.body).to match(/No configuration found/)
    end
  end
end

RSpec.describe 'Anonymous posting' do
  let(:req) { { site: 'test', path: '/', payload: { body: 'hello' } } }
  let(:response) { post("/api/comment", req) }
  let(:json) { JSON.parse(response.body, symbolize_names: true) }

  context 'is allowed' do
    before do
      add_site('test', private: false, anonymous: true, moderated: false)
    end

    it 'allows posting and assigns Anonymous as name' do
      expect(json[:comment]).to match(hash_including(id: 1, name: 'Anonymous', body: 'hello'))
      expect(json[:token]).to_not be_nil
    end
  end

  context 'is not allowed' do
    let(:site) { add_site('test', private: false, anonymous: false, moderated: false) }

    context 'an anonymous user' do
      before { site }

      it 'is not allowed to post' do
        expect(response.status).to eq 401
      end
    end

    context 'a signed user' do
      let(:s) { sign({ name: 'some user', moderator: false }, site) }
      let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'hello' } } }

      it 'is allowed to post' do
        expect(json[:comment]).to match(hash_including(id: 1, name: 'some user', body: 'hello'))
      end
    end

    context 'a signed user without a name' do
      let(:s) { sign({ moderator: false }, site) }
      let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'hello' } } }

      it 'is not allowed to post' do
        expect(response.status).to eq 400
        expect(response.body).to match(/User name is required for non-anonymous sites/)
      end
    end

    context 'a logged in moderator' do
      let(:s) { sign({ name: 'moderator', moderator: true }, site) }
      let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'hello' } } }

      before { site }

      it 'is allowed to post' do
        expect(json[:comment]).to match(hash_including(id: 1, name: 'moderator', body: 'hello'))
      end
    end
  end
end

RSpec.describe 'Moderation' do
  let(:req) { { site: 'test', path: '/', payload: { body: 'hello' } } }
  let(:response) { post("/api/comment", req) }
  let(:json) { JSON.parse(response.body, symbolize_names: true) }
  let(:moderated) { false }
  let(:site) { add_site('test', private: false, anonymous: true, moderated:) }

  before { site }

  context 'disabled' do
    it 'allows comments straight away' do
      expect(json[:comment]).to match(hash_including(id: 1, name: 'Anonymous', body: 'hello', reviewed: true))
    end
  end

  context 'enabled' do
    let(:moderated) { true }

    it 'requires comments to be reviewed' do
      expect(json[:comment]).to match(hash_including(id: 1, name: 'Anonymous', body: 'hello', reviewed: false))
    end

    context 'a signed user comment' do
      let(:s) { sign({ name: 'some user', moderator: false }, site) }
      let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'hello' } } }

      it 'must be reviewed' do
        expect(json[:comment]).to match(hash_including(id: 1, name: 'some user', body: 'hello', reviewed: false))
      end
    end

    context 'a signed user that is the OP' do
      let(:s) { sign({ name: 'some user', moderator: false, op: true }, site) }
      let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'hello' } } }

      it 'is automatically reviewed' do
        expect(json[:comment]).to match(hash_including(id: 1, name: 'some user', body: 'hello', reviewed: true))
      end
    end

    context 'a signed moderator' do
      let(:s) { sign({ name: 'moderator', moderator: true }, site) }
      let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'hello' } } }

      it 'must be reviewed' do
        expect(json[:comment]).to match(hash_including(id: 1, name: 'moderator', body: 'hello', reviewed: true))
      end
    end
  end
end

RSpec.describe 'Commenter name' do
  let(:req) { { site: 'test', path: '/', payload: { name: 'Commenter', body: 'hello' } } }
  let(:response) { post("/api/comment", req) }
  let(:json) { JSON.parse(response.body, symbolize_names: true) }
  let(:site) { add_site('test', private: false, anonymous: true, moderated: false) }

  before { site }

  context 'an anonymous user' do
    it 'can set a name' do
      expect(json[:comment]).to match(hash_including(id: 1, name: 'Commenter', body: 'hello'))
    end

    context 'without a name' do
      let(:req) { { site: 'test', path: '/', payload: { body: 'hello' } } }

      it 'defaults to Anonymous' do
        expect(json[:comment]).to match(hash_including(id: 1, name: 'Anonymous', body: 'hello'))
      end
    end

    context 'with a blank name' do
      let(:req) { { site: 'test', path: '/', payload: { name: '   ', body: 'hello' } } }

      it 'defaults to Anonymous' do
        expect(json[:comment]).to match(hash_including(id: 1, name: 'Anonymous', body: 'hello'))
      end
    end
  end

  context 'a signed user' do
    let(:s) { sign({ name: 'signed user', moderator: false }, site) }
    let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { name: 'yohoho', body: 'hello' } } }

    it 'cannot set their name' do
      expect(json[:comment]).to match(hash_including(id: 1, name: 'signed user', body: 'hello'))
    end
  end

  context 'a signed user wihout a name' do
    let(:s) { sign({ moderator: false }, site) }
    let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { body: 'hello' } } }

    it 'defaults to Anonymous' do
      expect(json[:comment]).to match(hash_including(id: 1, name: 'Anonymous', body: 'hello'))
    end
  end

  context 'a signed moderator' do
    let(:s) { sign({ name: 'moderator', moderator: true }, site) }
    let(:req) { { site: 'test', path: '/', user: s.first, signature: s.last, payload: { name: 'yohoho', body: 'hello' } } }

    it 'cannot set their name' do
      expect(json[:comment]).to match(hash_including(id: 1, name: 'moderator', body: 'hello'))
    end
  end
end
