RSpec.describe 'Getting config' do
  context 'with non-existing site' do
    it 'returns bad response' do
      response = post("/api/config", { site: "test", path: "/" })

      expect(response.status).to eq(400)
      expect(response.body).to match(/No configuration found/)
    end
  end

  context 'with an existing site' do
    before { add_site('test') }

    it 'returns the config' do
      response = post("/api/config", { site: "test", path: "/" })
      expect(response.status).to eq(200)

      expect(JSON.parse(response.body, symbolize_names: true)).to eq({
        anonymous: false,
        moderated: true,
        locked: false
      })
    end
  end
end

RSpec.describe 'Locking' do
  let(:site) { add_site('test', private: false, anonymous: true, moderated: false) }
  let(:s1) { sign({ name: 'some user' }, site) }
  let(:s2) { sign({ name: 'moderator', moderator: true }, site) }
  let(:req) { { site: "test", path: "/" } }
  let(:response) { patch("/api/pages", req) }

  before { site }

  context 'an anonymous user' do
    it 'returns an error' do
      expect(response.status).to eq(401)
    end
  end

  context 'a signed non-moderator' do
    let(:req) { { site: "test", path: "/", user: s1.first, signature: s1.last } }

    it 'returns an error' do
      expect(response.status).to eq(403)
    end
  end

  context 'a moderator' do
    let(:req) { { site: "test", path: "/", user: s2.first, signature: s2.last } }

    it 'returns an error' do
      expect(response.status).to eq(200)
    end
  end
end
