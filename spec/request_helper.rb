module RequestHelper
  %w[post patch put].each do |method|
    define_method method do |endpoint, body|
      Faraday.send(
        method,
        "http://localhost:6353#{endpoint}",
        body.to_json,
        { 'Content-Type' => 'application/json' }
      )
    end
  end

  def delete(endpoint, body)
    Faraday.new("http://localhost:6353").delete(endpoint) do |req|
      req.body = body.to_json
      req.headers['Content-Type'] = 'application/json'
    end
  end
end
