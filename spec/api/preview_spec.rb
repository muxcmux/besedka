RSpec.describe 'Previewing markdown' do
  context 'with non-existing site' do
    it 'returns bad response' do
      response = post("/api/preview", { site: "test", path: "/" })

      expect(response.status).to eq(400)
      expect(response.body).to match(/No configuration found/)
    end
  end

  context 'with an existing site' do
    before { add_site('test', private: false, anonymous: true) }

    context 'without a body to preview' do
      it 'returns an error' do
        response = post("/api/preview", { site: "test", path: "/" })
        expect(response.status).to eq(422)
        expect(response.body).to match(/Missing body/)
      end
    end

    context 'with a body to preview' do
      let(:payload) do
        "This __is__ a comment with a link to http://google.com :+1"
      end

      let(:md) do
        '<p>This <strong>is</strong> a comment with a link to <a href="http://google.com">http://google.com</a> :+1</p>'
      end

      it 'returns the preview' do
        response = post("/api/preview", { site: "test", path: "/", payload: })
        expect(response.status).to eq(200)
        expect(response.body).to eq(md)
      end
    end
  end
end

