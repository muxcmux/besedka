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
