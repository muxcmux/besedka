RSpec.describe 'Listing comments' do
  let(:site) { 'test.com' }
  let(:private) { false }
  let(:moderated) { false }
  let(:anonymous) { true }
  let(:comments_per_page) { 5 }
  let(:replies_per_comment) { 3 }

  def get_comments
    JSON.parse(post('/api/comments', { site:, path: '/' }).body, symbolize_names: true)
  end

  before(:each) { add_site(site, private:, anonymous:, moderated:, comments_per_page:, replies_per_comment:) }

  describe 'on a public page' do
    before(:each) do
      5.times do |i|
        post(
          '/api/comment',
          { site:, path: '/', payload: { body: "hello world #{i}" } }
        )
      end
    end

    context 'with a single page' do
      it 'displays a list of comments with the newest on top' do
        get_comments
        expect(get_comments).to match(
          hash_including(
            comments: [
              hash_including(id: 5, name: 'Anonymous', body: 'hello world 4', thread: { cursor: nil, replies: [] }),
              hash_including(id: 4, name: 'Anonymous', body: 'hello world 3', thread: { cursor: nil, replies: [] }),
              hash_including(id: 3, name: 'Anonymous', body: 'hello world 2', thread: { cursor: nil, replies: [] }),
              hash_including(id: 2, name: 'Anonymous', body: 'hello world 1', thread: { cursor: nil, replies: [] }),
              hash_including(id: 1, name: 'Anonymous', body: 'hello world 0', thread: { cursor: nil, replies: [] })
            ],
            cursor: nil,
            total: 5
          )
        )
      end
    end

    context 'with multiple pages' do
      let(:comments_per_page) { 2 }

      it 'displays the newest comments an a cursor to the next page' do
        comments = get_comments
        expect(comments).to match(
          hash_including(
            comments: [
              hash_including(id: 5, name: 'Anonymous', body: 'hello world 4', thread: { cursor: nil, replies: [] }),
              hash_including(id: 4, name: 'Anonymous', body: 'hello world 3', thread: { cursor: nil, replies: [] })
            ],
            total: 5
          )
        )
        expect(comments[:cursor]).to_not be nil
      end
    end
  end
end
