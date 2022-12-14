RSpec.describe 'logging in' do
  let(:site) { add_site('test', private: false, anonymous: true, moderated: false) }
  let(:moderator) { add_moderator }
  let(:response) { post('/api/login', req) }
  let(:sid) { JSON.parse(response.body)['sid'] }

  before { site }

  context 'with invalid creds' do
    let(:req) { { name: 'wrong', password: 'wrong' } }

    it 'returns an error' do
      expect(response.status).to eq 401
    end
  end

  context 'with correct creds' do
    let(:req) { moderator }

    it 'returns the sid' do
      expect(sid).to_not be_nil
    end
  end
end
